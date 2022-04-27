require('dotenv').config();
const Web3 = require('web3');
const chai = require('chai');
chai.use(require('chai-as-promised'));
const { BigNumber } = require('@ethersproject/bignumber');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const wsProvider = new WsProvider(process.env.WS_API);
const web3 = new Web3(process.env.RPC_API);
const { Keyring } = require('@polkadot/api');

var ERC20ABI = require('../build/contracts/GAKI.json');
const { u8aToHex } = require('@polkadot/util');

async function add_additional_gas(contract, address) {
    const gas_limit = await contract.estimateGas({ from: address });

    const additional_gas = BigNumber.from(gas_limit.toString())
        .mul(BigNumber.from(process.env.ADD_GAS_LIMIT.toString())).div(BigNumber.from("100"));
    return BigNumber.from(gas_limit.toString()).add(additional_gas).toString();
}

async function create_new_contract(account) {
    const arguments = [
    ];
    const contract = new web3.eth.Contract(ERC20ABI.abi);
    const contract_data = await contract.deploy({
        data: ERC20ABI.bytecode,
        arguments: arguments
    });
    const nonce = await web3.eth.getTransactionCount(account.address, "pending");
    const options = {
        data: contract_data.encodeABI(),
        gas: await add_additional_gas(contract_data, account.address),
        gasPrice: await web3.eth.getGasPrice(),
        nonce,
    };
    const signed = await web3.eth.accounts.signTransaction(options, account.privateKey);
    const receipt = await web3.eth.sendSignedTransaction(signed.rawTransaction);
    return receipt;
}

async function transfer_erc20(token_address, account, target, amount) {
    const erc20_contract = new web3.eth.Contract(ERC20ABI.abi, token_address);
    const contract = await erc20_contract.methods.transfer(target, amount);

    let gas_limit = await add_additional_gas(contract, account.address);

    const options = {
        to: token_address,
        data: contract.encodeABI(),
        gas: gas_limit,
        gasPrice: await web3.eth.getGasPrice()
    };

    const signed = await web3.eth.accounts.signTransaction(options, account.privateKey);
    const receipt = await web3.eth.sendSignedTransaction(signed.rawTransaction);
    return receipt;
}

/// map account to alice
async function proof_address_mapping(evm_account, sub_account) {
    const api = await ApiPromise.create({ provider: wsProvider });

    let signature;
    {
        const data = u8aToHex(sub_account.publicKey, undefined, false);
        let message = `Bond Gafi Network account:${data}`;
        let sign_data = evm_account.sign(message);
        signature = sign_data.signature;
    }
    const txExecute = api.tx.addressMapping.bond(
        signature,
        evm_account.address,
        false
    );
    const unsub = await txExecute
        .signAndSend(sub_account);
    return unsub;
}

async function join_pool(sub_account, service) {
    const api = await ApiPromise.create({ provider: wsProvider });

    const txExecute = api.tx.pool.join(
        service
    );

    const unsub = await txExecute
        .signAndSend(sub_account);
    return unsub;
}

async function leave_pool(sub_account) {
    const api = await ApiPromise.create({ provider: wsProvider });

    const txExecute = api.tx.pool.leave();

    const unsub = await txExecute
        .signAndSend(sub_account);
    return unsub;
}

async function create_pool(sub_account, arguments) {
    const api = await ApiPromise.create({ provider: wsProvider });

    const txExecute = api.tx.sponsoredPool.createPool(arguments.targets, arguments.value, arguments.discount, arguments.txLimit);

    const unsub = await txExecute
        .signAndSend(sub_account);
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
