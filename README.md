# Gafi Network Node

The decentralized blockchain platform that built for blockchain games and high-frequency applications. Applications running on top of Gafi Network will never bother about the security, transaction fee, malicious accounts, and many other bad factors. They can just focus on gameplay, game experience, or application logic. The Gafi Network supports EVM (Ethereum Virtual Machine) that projects deploy from Ethereum or Binance Smart Chain to Gafi Network with little or no change.

## Getting Started

Follow the steps below to get started with the Node Template, or get it up and running right from
your browser in just a few clicks using
the [Substrate Playground](https://docs.substrate.io/playground/) :hammer_and_wrench:


### Rust Setup

First, complete the [basic Rust setup instructions](./docs/rust-setup.md).

### Test
  #### Test pallet functionalities
  ```sh
  make test
  ```
  #### [Client tests](https://wiki.gafi.network/build/how-to-guides/how-to-run-client-tests)


### Run

```sh
make run-dev
```

### Build

```sh
make build
```

### Benchmarking

```sh
make benchmark
```

### Docs

https://wiki.gafi.network


### Connect with Polkadot-JS Apps Front-end

Once the node template is running locally, you can connect it with **Polkadot-JS Apps** front-end
to interact with your chain. [Click
here](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) connecting the Apps to your
local node template.


### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command
(`cargo build --release && ./target/release/node-template --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/node-template --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/node-template purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```
