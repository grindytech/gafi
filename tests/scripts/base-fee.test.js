require('dotenv').config();
const { assert, equal } = require('assert');
const Web3 = require('web3');
const web3 = new Web3(process.env.RPC_API);
const axios = require('axios');
const chai = require('chai');
const cryptojs = require('crypto-js');
const expect = chai.expect;
chai.use(require('chai-as-promised'));
var randomBytes = require('randombytes');
const { BigNumber } = require('@ethersproject/bignumber');
const { ethers, keccak256 } = require("ethers");
const { arrayify, BytesLike } = require("@ethersproject/bytes");
const utils = require('./utils');

const test1 = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
const test2 = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);

var ERC20_Address;

describe('Contract', () => {

    it('base transaction fee create token erc20', async () => {
        let before_balance = await web3.eth.getBalance(test1.address);
        console.log("deploying...");
        const gaki_token = await utils.create_new_contract(test1);
        ERC20_Address = gaki_token.contractAddress;
        let after_balance = await web3.eth.getBalance(test1.address);
        console.log("create erc20 token fee: ", web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString()), "GAKI");
    }).timeout(3600000);

    it('base transaction fee by transfer erc20', async () => {
        let before_balance = await web3.eth.getBalance(test1.address);
        console.log("deploying...");
        let transfer_data = await  utils.transfer_erc20(ERC20_Address, test1, test2.address, "100000000000000" );
        let after_balance = await web3.eth.getBalance(test1.address);
        console.log("transfer erc20 token fee: ", web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString()), "GAKI");
    }).timeout(3600000);

})

