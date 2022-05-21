require('dotenv').config();
const Web3 = require('web3');
const chai = require('chai');
chai.use(require('chai-as-promised'));
const { BigNumber } = require('@ethersproject/bignumber');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/api');
const { customRequest, createAndFinalizeBlock, WS_PORT } = require('./context');

var ERC20ABI = require('../build/contracts/GAKI.json');
const { u8aToHex } = require('@polkadot/util');

async function add_additional_gas(contract, address) {
    const gas_limit = await contract.estimateGas({ from: address });

    const additional_gas = BigNumber.from(gas_limit.toString())
        .mul(BigNumber.from(process.env.ADD_GAS_LIMIT.toString())).div(BigNumber.from("100"));
    return BigNumber.from(gas_limit.toString()).add(additional_gas).toString();
}

async function create_new_contract(context, account) {
    const arguments = [
    ];
    const contract = new context.web3.eth.Contract(ERC20ABI.abi);
    const contract_data = await contract.deploy({
        data: ERC20ABI.bytecode,
        arguments: arguments
    });
    const nonce = await context.web3.eth.getTransactionCount(account.address, "pending");
    const options = {
        data: contract_data.encodeABI(),
        gas: await add_additional_gas(contract_data, account.address),
        gasPrice: await context.web3.eth.getGasPrice(),
        nonce,
    };

    const signed = await context.web3.eth.accounts.signTransaction(options, account.privateKey);
    const tx_hash = (await customRequest(context.web3, "eth_sendRawTransaction", [signed.rawTransaction])).result;
    await createAndFinalizeBlock(context.web3);
    const receipt = (await customRequest(context.web3, "eth_getTransactionReceipt", [tx_hash]));
    return receipt.result;
}

async function transfer_erc20(context, token_address, account, target, amount) {
    const erc20_contract = new context.web3.eth.Contract(ERC20ABI.abi, token_address);
    const contract = await erc20_contract.methods.transfer(target, amount);

    let gas_limit = await add_additional_gas(contract, account.address);

    const options = {
        to: token_address,
        data: contract.encodeABI(),
        gas: gas_limit,
        gasPrice: await context.web3.eth.getGasPrice()
    };

    const signed = await context.web3.eth.accounts.signTransaction(options, account.privateKey);
    const tx_hash = (await customRequest(context.web3, "eth_sendRawTransaction", [signed.rawTransaction])).result;
    return tx_hash;
}

/// map account to alice
async function proof_address_mapping(context, evm_account, sub_account) {
    const wsProvider = new WsProvider(`ws://127.0.0.1:${WS_PORT}`);
    const api = await ApiPromise.create({ provider: wsProvider });

    let signature;
    {
        const data = u8aToHex(sub_account.publicKey, undefined, false);
        let message = `Bond Gafi Network account:${data}`;
        let sign_data = evm_account.sign(message);
        signature = sign_data.signature;
    }
    const txExecute = api.tx.proofAddressMapping.bond(
        signature,
        evm_account.address,
        false
    );
    const unsub = await txExecute
        .signAndSend(sub_account);
    await createAndFinalizeBlock(context.web3);
    return unsub;
}

async function join_pool(context, sub_account, service) {
    const wsProvider = new WsProvider(`ws://127.0.0.1:${WS_PORT}`);
    const api = await ApiPromise.create({ provider: wsProvider });
    const txExecute = api.tx.pool.join(
        service
    );

    const unsub = await txExecute
        .signAndSend(sub_account);
    await createAndFinalizeBlock(context.web3);
    return unsub;
}

async function leave_pool(context, sub_account) {
    const wsProvider = new WsProvider(`ws://127.0.0.1:${WS_PORT}`);
    const api = await ApiPromise.create({ provider: wsProvider });
    const txExecute = api.tx.pool.leave();

    const unsub = await txExecute
        .signAndSend(sub_account);
    await createAndFinalizeBlock(context.web3);
    return unsub;
}

async function create_pool(context, sub_account, arguments) {
    const wsProvider = new WsProvider(`ws://127.0.0.1:${WS_PORT}`);
    const api = await ApiPromise.create({ provider: wsProvider });

    const txExecute = api.tx.sponsoredPool.createPool(arguments.targets, arguments.value, arguments.discount, arguments.txLimit);
    const unsub = await txExecute
        .signAndSend(sub_account);
    await createAndFinalizeBlock(context.web3);
    return unsub;
}

module.exports = {
    add_additional_gas,
    create_new_contract,
    transfer_erc20,
    proof_address_mapping,
    join_pool,
    leave_pool,
    create_pool,
}
