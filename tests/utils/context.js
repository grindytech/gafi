// convert ts-test from Frontier https://github.com/paritytech/frontier/blob/master/ts-tests/tests/util.ts

const Web3 = require("web3");
const { ethers } = require("ethers");
const { JsonRpcResponse } = require("web3-core-helpers");
const { spawn, ChildProcess, exec } = require("child_process");
const { WsProvider } = require("@polkadot/api");

const RPC_PORT = 9933;
const WS_PORT = 9944;

const DISPLAY_LOG = process.env.FRONTIER_LOG || false;
const FRONTIER_LOG = process.env.FRONTIER_LOG || "info";
const FRONTIER_BUILD = process.env.FRONTIER_BUILD || "release";

const BINARY_PATH = `../target/${FRONTIER_BUILD}/gafi-node`;
const SPAWNING_TIME = 60000;

async function customRequest(web3, method, params) {
	return new Promise((resolve, reject) => {
		web3.currentProvider.send(
			{
				jsonrpc: "2.0",
				id: 1,
				method,
				params,
			},
			(error, result) => {
				if (error) {
					reject(
						`Failed to send custom request (${method} (${params.join(",")})): ${error.message || error.toString()
						}`
					);
				}
				resolve(result);
			}
		);
	});
}

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
async function createAndFinalizeBlock(web3) {
	const response = await customRequest(web3, "engine_createBlock", [true, true, null]);
	if (!response.result) {
		throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
	}
	await new Promise(resolve => setTimeout(() => resolve(), 500));
}

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
async function createAndFinalizeBlockNowait(web3) {
	const response = await customRequest(web3, "engine_createBlock", [true, true, null]);
	if (!response.result) {
		throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
	}
}

async function startFrontierNode(provider) {
	var web3;
	if (!provider || provider == 'http') {
		web3 = new Web3(`http://localhost:${RPC_PORT}`);
	}

	const cmd = BINARY_PATH;
	const args = [
		`--chain=dev`,
		`--validator`,
		`--execution=Native`,
		`--no-telemetry`,
		`--no-prometheus`,
		`--sealing=Manual`,
		`--no-grandpa`,
		`--force-authoring`,
		`-l${FRONTIER_LOG}`,
		`--rpc-port=${RPC_PORT}`,
		`--ws-port=${WS_PORT}`,
		`--tmp`,
	];

	const binary = spawn(cmd, args);

	binary.on("error", (err) => {
		if (err.errno == "ENOENT") {
			console.error(
				`\x1b[31mMissing Frontier binary (${BINARY_PATH}).\nPlease compile the Frontier project:\ncargo build\x1b[0m`
			);
		} else {
			console.error(err);
		}
		process.exit(1);
	});

	const binaryLogs = [];
	await new Promise((resolve) => {
		const timer = setTimeout(() => {
			console.error(`\x1b[31m Failed to start Frontier Template Node.\x1b[0m`);
			console.error(`Command: ${cmd} ${args.join(" ")}`);
			console.error(`Logs:`);
			console.error(binaryLogs.map((chunk) => chunk.toString()).join("\n"));
			process.exit(1);
		}, SPAWNING_TIME - 2000);

		const onData = async (chunk) => {
			if (DISPLAY_LOG) {
				console.log(chunk.toString());
			}
			binaryLogs.push(chunk);
			if (chunk.toString().match(/Manual Seal Ready/)) {
				if (!provider || provider == "http") {
					// This is needed as the EVM runtime needs to warmup with a first call
					await web3.eth.getChainId();
				}

				clearTimeout(timer);
				if (!DISPLAY_LOG) {
					binary.stderr.off("data", onData);
					binary.stdout.off("data", onData);
				}
				// console.log(`\x1b[31m Starting RPC\x1b[0m`);
				resolve();
			}
		};
		binary.stderr.on("data", onData);
    binary.stdout.on("data", onData);
	});

	if (provider == 'ws') {
		web3 = new Web3(`ws://localhost:${WS_PORT}`);
	}

	let ethersjs = new ethers.providers.StaticJsonRpcProvider(`http://localhost:${RPC_PORT}`, {
		chainId: 1337,
		name: "frontier-dev",
  });

	return { web3, binary, ethersjs };
}

function describeWithFrontier(title, cb, provider) {
	describe(title, () => {
		let context = { web3: null, ethersjs: null, wsProvider: null };
		let binary;
		// Making sure the Frontier node has started
		before("Starting Frontier Test Node", async function () {
			this.timeout(SPAWNING_TIME);
			const init = await startFrontierNode(provider);
			context.web3 = init.web3;
      context.ethersjs = init.ethersjs;
      context.wsProvider = new WsProvider(`ws://127.0.0.1:${WS_PORT}`);
			binary = init.binary;
		});

		after(async function () {
			//console.log(`\x1b[31m Killing RPC\x1b[0m`);
      await context.wsProvider?.disconnect();
      binary.kill();

		});

		cb(context);
	});
}

module.exports = {
	describeWithFrontier,
	RPC_PORT,
	WS_PORT,
	customRequest,
	createAndFinalizeBlock,
}
