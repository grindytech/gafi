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
    web3.eth.accounts.privateKeyToAccount("bcf293ba01f30136a0d861e2ffe76c17ea6fd728bfbb2da6b09a6994846057fe"), // test 1
    web3.eth.accounts.privateKeyToAccount("943272b0eaa0392e251cba3d1525a9b518b2c47ffb934e0266dab32b40826f22"), // test 2
    web3.eth.accounts.privateKeyToAccount("bcf293ba01f30136a0d861e2ffe76c17ea6fd728bfbb2da6b09a6994846057fe"), // test 1
]

const ENCODEDS = [
    "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d", // alice
    "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48", // bob
    "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48", // bob
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

