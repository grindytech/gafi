require('dotenv').config();
const Web3 = require('web3');
const web3 = new Web3(process.env.RPC_API);
const chai = require('chai');
chai.use(require('chai-as-promised'));
const { BigNumber } = require('@ethersproject/bignumber');
const utils = require('./utils');

const test1 = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
const test2 = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
const test3 = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_3);

const TEST_COUNT = process.env.TEST_COUNT;

describe('Contract', () => {

    it('show total fee spent when ouside the pool', async () => {
        let admin = test1;
        let before_balance = await web3.eth.getBalance(admin.address);
        console.log("before_balance: ", before_balance);
        console.log("deploying...");
        for (let i = 0; i < TEST_COUNT; i++) {
            let receipt = await utils.create_new_contract(admin);
        }
        let after_balance = await web3.eth.getBalance(admin.address);
        console.log("after_balance: ", after_balance);

        console.log("total_cost: ", web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString()), "GAKI");
    }).timeout(3600000);

    it('show total fee spent when inside the pool', async () => {
        let admin = test2;
        let before_balance = await web3.eth.getBalance(admin.address);
        console.log("before_balance: ", before_balance);
        console.log("deploying...");

        let receipt = await utils.create_new_contract(admin);
        
        let after_balance = await web3.eth.getBalance(admin.address);
        console.log("after_balance: ", after_balance);

        console.log("total_cost: ", web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString()), "GAKI");
    }).timeout(3600000);
})

