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

const admin = web3.eth.accounts.privateKeyToAccount("5240c93f837385e95742426ebc0dc49bbbeded5a9aaec129ac9de1754ca98ccb");

async function add_additional_gas(contract, address) {
    const gas_limit = await contract.estimateGas({ from: address });
    const additional_gas = BigNumber.from(gas_limit.toString()).mul("50").div("100");
    return BigNumber.from(gas_limit.toString()).add(additional_gas).toString();
}
const ALICE = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
const ALICE_ENCODED = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

describe('Contract', () => {

    it("it should get bind arguments data", async () => {
        const test_message = `Bind Aurora Network account:${ALICE_ENCODED}`;
        let message; // keccak256 of ALICE address
        let signature; // signature of sign the message
        let address; // user address
        {
            let sign_data = admin.sign(test_message);
            console.log("signature: ", sign_data.signature);
        }

    })

    it('it should create new ERC20 token', async () => {

        for (let i = 0; i < 10; i++) {
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
    }).timeout(500000);



})

