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

console.log("account: ", admin);

async function add_additional_gas(contract, address) {
    const gas_limit = await contract.estimateGas({ from: address });
    const additional_gas = BigNumber.from(gas_limit.toString()).mul("50").div("100");
    return BigNumber.from(gas_limit.toString()).add(additional_gas).toString();
}
const ALICE = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
const ALICE_BYTES = [212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189,
    4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125];
const ALICE_HASH = [159, 1, 180, 141, 164, 34, 185, 74, 191, 225, 106, 29,
    150, 33, 221, 230, 54, 222, 9, 139, 67, 239, 176, 250, 155, 97, 250, 207, 227, 40, 249, 157];

const MESSAGE = [159, 1, 180, 141, 164, 34, 185, 74, 191, 225, 106, 29, 150, 33, 221, 230, 54, 222,
    9, 139, 67, 239, 176, 250, 155, 97, 250, 207, 227, 40, 249, 157];

const SIGNATURE = [174, 245, 232, 212, 128, 155, 118, 64, 129, 225, 22, 140, 100, 179, 16, 222,
    138, 184, 122, 27, 37, 57, 129, 102, 57, 55, 77, 19, 15, 35, 117, 204, 102, 86, 153, 119,
    214, 131, 250, 193, 180, 52, 105, 220, 184, 8, 12, 77, 60, 33, 193, 28, 75, 248, 188, 32,
    151, 249, 210, 159, 130, 27, 5, 181, 0];

function toHexString(byteArray) {
    //const chars = new Buffer(byteArray.length * 2);
    const chars = new Uint8Array(byteArray.length * 2);
    const alpha = 'a'.charCodeAt(0) - 10;
    const digit = '0'.charCodeAt(0);

    let p = 0;
    for (let i = 0; i < byteArray.length; i++) {
        let nibble = byteArray[i] >>> 4;
        chars[p++] = nibble > 9 ? nibble + alpha : nibble + digit;
        nibble = byteArray[i] & 0xF;
        chars[p++] = nibble > 9 ? nibble + alpha : nibble + digit;
    }

    //return chars.toString('utf8');
    return String.fromCharCode.apply(null, chars);
}

describe('Contract', () => {

    it("it should get bind arguments data", async () => {
        const test_message = "Pay RUSTs to the TEST account:c03547727776614546357a58623236467a397263517044575335374374455248704e6568584350634e6f48474b75745159";
        let message; // keccak256 of ALICE address
        let signature; // signature of sign the message
        let address; // user address
        {
            let address = web3.utils.utf8ToHex(ALICE);
            console.log("address: ", address);
          let signature = admin.sign(test_message);
          console.log("signature: ", signature);
        }

    })

    it('it should create new ERC20 token', async () => {
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

    }).timeout(30000);



})

