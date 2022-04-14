require('dotenv').config();
const Web3 = require('web3');
const web3 = new Web3(process.env.RPC_API);
const chai = require('chai');
chai.use(require('chai-as-promised'));
const { BigNumber } = require('@ethersproject/bignumber');

var ERC20ABI = require('../build/contracts/GAKI.json');

async function add_additional_gas(contract, address) {

    const gas_limit = await contract.estimateGas({ from: address });

    const additional_gas = BigNumber.from(gas_limit.toString())
        .mul(BigNumber.from(process.env.ADD_GAS_LIMIT.toString())).div(BigNumber.from("100"));
    return BigNumber.from(gas_limit.toString()).add(additional_gas).toString();
}

async function create_new_contract(account) {
    const arguments = [
        "1000000000000000000000000000" // 1B
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


module.exports = {
    add_additional_gas,
    create_new_contract,
    transfer_erc20,
}
