require('dotenv').config();
const { assert, equal } = require('assert');
const Web3 = require('web3');
const web3 = new Web3(process.env.RPC_API);
const axios = require('axios');
const chai = require('chai');
const cryptojs = require('crypto-js');
const expect = chai.expect;
chai.use(require('chai-as-promised'));

const ACCOUNTS = [
    web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1),
    web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2),
    web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_3),
]

const ENCODEDS = [
    "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48", // bob
    "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48", // bob
    "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d", // alice
]

describe('Contract', () => {
    it("it should get bind arguments data", async () => {
        for (let i = 0; i < ACCOUNTS.length; i++) {
            const test_message = `Bond Gafi Network account:${ENCODEDS[i]}`;
            let sign_data = ACCOUNTS[i].sign(test_message);
            console.log({
                "address: ": ACCOUNTS[i].address,
                "encode: ": ENCODEDS[i],
                "sig: ": sign_data.signature,
            })
        }
    })
})
