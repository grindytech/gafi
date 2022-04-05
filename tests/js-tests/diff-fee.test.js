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

const test1 = web3.eth.accounts.privateKeyToAccount("bcf293ba01f30136a0d861e2ffe76c17ea6fd728bfbb2da6b09a6994846057fe");
const test2 = web3.eth.accounts.privateKeyToAccount("943272b0eaa0392e251cba3d1525a9b518b2c47ffb934e0266dab32b40826f22");
const test3 = web3.eth.accounts.privateKeyToAccount("0c6a0445624c4f0e51feeb56eb86e286adb9f94ff486c53fab63c36844602749");


async function add_additional_gas(contract, address) {
    const gas_limit = await contract.estimateGas({ from: address });
    const additional_gas = BigNumber.from(gas_limit.toString()).mul("50").div("100");
    return BigNumber.from(gas_limit.toString()).add(additional_gas).toString();
}

const TX_COUNT = 20;

describe('Contract', () => {
    it('show total fee spent when ouside the pool', async () => {
        let admin = test1;
        let before_balance = await web3.eth.getBalance(admin.address);
        console.log("before_balance: ", before_balance);
        console.log("deploying...");
        for (let i = 0; i < TX_COUNT; i++) {
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
            // console.log("receipt: ", receipt);
        }
        let after_balance = await web3.eth.getBalance(admin.address);
        console.log("after_balance: ", after_balance);

        console.log("total_cost: ", BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString());
    }).timeout(3600000);

    it('show total fee spent when inside the pool', async () => {
        let admin = test2;
        let before_balance = await web3.eth.getBalance(admin.address);
        console.log("before_balance: ", before_balance);
        console.log("deploying...");

        for (let i = 0; i < TX_COUNT; i++) {
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
        }
        let after_balance = await web3.eth.getBalance(admin.address);
        console.log("after_balance: ", after_balance);

        console.log("total_cost: ", BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString());
    }).timeout(3600000);
})

