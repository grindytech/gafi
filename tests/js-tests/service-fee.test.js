require('dotenv').config();
const Web3 = require('web3');
const chai = require('chai');
chai.use(require('chai-as-promised'));
const { BigNumber } = require('@ethersproject/bignumber');
const utils = require('../utils/util');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/api');
const keyring = new Keyring({ type: 'sr25519' });
var assert = require('assert');
const { describeWithFrontier, WS_PORT, RPC_PORT } = require('../utils/context');
const { step } = require("mocha-steps");

function delay(interval) {
    return step(`should delay ${interval}`, done => {
        setTimeout(() => done(), interval)
    }).timeout(interval + 100)
}

var nomal_fee;

function percentage_of(oldNumber, newNumber) {
    return (1 - (oldNumber / newNumber)) * 100
}

function create_erc20_token_circle(context, ticket, expect_rate, tx_limit) {
    step('leave pool works', async () => {
        const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.leave_pool(context, alice);
    }).timeout(10000);

    step(`join pool works`, async () => {
        const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.join_pool(context, alice, ticket);
    }).timeout(10000);

    step('Discount with tx limit works', async () => {
        let account2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        let count = 0;
        for (let i = 0; i < 12; i++) {
            let before_balance = await context.web3.eth.getBalance(account2.address);
            let receipt = await utils.create_new_contract(context, account2);
            let after_balance = await context.web3.eth.getBalance(account2.address);
            let staking_fee = context.web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
            let rate = percentage_of(staking_fee, nomal_fee);

            if (count < tx_limit) {
                assert.equal(Math.round(rate), expect_rate);
            } else {
                assert.notEqual(Math.round(rate), expect_rate);
            }
            count++;
        }
    }).timeout(10000);
}

describeWithFrontier("Upfront and Staking Pool Fee", (context) => {

    step('show total fee spent when ouside the pool', async () => {
        let account = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        let before_balance = await context.web3.eth.getBalance(account.address);

        let receipt = await utils.create_new_contract(context, account);

        let after_balance = await context.web3.eth.getBalance(account.address);
        nomal_fee = context.web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
        console.log("Normal fee: ", nomal_fee);
    }).timeout(10000);

    step('step should mapping addresses', async () => {
        let account2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.proof_address_mapping(context, account2, alice);
    }).timeout(10000);

    create_erc20_token_circle(context, { Staking: "Basic" }, 30, 10);
    create_erc20_token_circle(context, { Staking: "Medium" }, 50, 10);
    create_erc20_token_circle(context, { Staking: "Advance" }, 70, 10);
    
    create_erc20_token_circle(context, { Upfront: "Basic" }, 30, 10);
    create_erc20_token_circle(context, { Upfront: "Medium" }, 50, 10);
    create_erc20_token_circle(context, { Upfront: "Advance" }, 70, 10);

})
