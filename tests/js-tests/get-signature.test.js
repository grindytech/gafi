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

const ACCOUNTS = [
    web3.eth.accounts.privateKeyToAccount("5240c93f837385e95742426ebc0dc49bbbeded5a9aaec129ac9de1754ca98ccb"), // test 1
    web3.eth.accounts.privateKeyToAccount("b109fbfdbb77af91889ff90fa79aa8b15fd39a18b4f761253ab4e4ab4faa1717"), // test 2
    web3.eth.accounts.privateKeyToAccount("db971a6ab5242bc883f958f8098e2497b2b6d670a02c175a66a74c162f77596b"), // test 3
]

const ENCODEDS = [
    "4dffba8214fbcc626ea93064efddbbb1a6c2ca36fdae1c165d7626ffd6b53ad2", // ben 1
    "32dffb31e24d8bdeb615dea72936c7ac730ebcd1690c485c16cf0e65200e71bd", // bench 2
    "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d", // alice
]

describe('Contract', () => {
    it("it should get bind arguments data", async () => {
        for (let i = 0; i < ACCOUNTS.length; i++) {
            const test_message = `Bond Aurora Network account:${ENCODEDS[i]}`;
            let sign_data = ACCOUNTS[i].sign(test_message);
            console.log({
                "address: ": ACCOUNTS[i].address,
                "encode: ": ENCODEDS[i],
                "sig: ": sign_data.signature,
            })
        }
    })
})

