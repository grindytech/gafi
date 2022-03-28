require('dotenv').config();
const { assert, equal } = require('assert');
const RPC = "http://127.0.0.1:9933";
const Web3 = require('web3');
const web3 = new Web3(RPC);
const axios = require('axios');
const chai = require('chai');
const cryptojs = require('crypto-js');
const expect = chai.expect;
chai.use(require('chai-as-promised'));
var randomBytes = require('randombytes');
const { BigNumber } = require('@ethersproject/bignumber');
const { ethers, keccak256 } = require("ethers");
const { arrayify, BytesLike } = require("@ethersproject/bytes");

var ERC20ABI = require('../build/contracts/ERC20.json');

const test1 = web3.eth.accounts.privateKeyToAccount("5240c93f837385e95742426ebc0dc49bbbeded5a9aaec129ac9de1754ca98ccb");
const test2 = web3.eth.accounts.privateKeyToAccount("b109fbfdbb77af91889ff90fa79aa8b15fd39a18b4f761253ab4e4ab4faa1717");

let admin = test1;


async function add_additional_gas(contract, address) {
    const gas_limit = await contract.estimateGas({ from: address });
    const additional_gas = BigNumber.from(gas_limit.toString()).mul("50").div("100");
    return BigNumber.from(gas_limit.toString()).add(additional_gas).toString();
}

describe('Contract', () => {
    it('it should create new ERC20 token', async () => {
        
        let before_balance = await web3.eth.getBalance(admin.address);
        console.log("before_balance: ", before_balance);

        for (let i = 0; i < 30; i++) {
            const arguments = [

            ];
            const stake_contract = new web3.eth.Contract(ERC20ABI.abi);
            const contract_data = await stake_contract.deploy({
                data: ERC20ABI.bytecode,
                arguments: arguments
            });

            const options = {
                data: contract_data.encodeABI(),
                gas: await add_additional_gas(contract_data, admin.address),
                gasPrice: await web3.eth.getGasPrice()
            };
            const signed = await web3.eth.accounts.signTransaction(options, admin.privateKey);
            const receipt = await web3.eth.sendSignedTransaction(signed.rawTransaction);

            // console.log("hash: ", receipt.transactionHash);
            console.log(i);
        }
        let after_balance = await web3.eth.getBalance(admin.address);
        console.log("after_balance: ", after_balance);

        console.log("total_cost: ", before_balance - after_balance);
    }).timeout(3600000);
})

