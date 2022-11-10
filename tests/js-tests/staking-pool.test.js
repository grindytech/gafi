require('dotenv').config();
const Web3 = require('web3');
const chai = require('chai');
chai.use(require('chai-as-promised'));
const { BigNumber } = require('@ethersproject/bignumber');
const utils = require('../utils/util');
const { Keyring } = require('@polkadot/api');
const keyring = new Keyring({ type: 'sr25519' });
var assert = require('assert');
const { describeWithFrontier, RPC_PORT } = require('../utils/context');
const { step } = require("mocha-steps");

const STAKING_BASIC_ID = "0x0000000000000000000000000000000000000000000000000000000000000000";
const STAKING_MEDIUM_ID = "0x0101010101010101010101010101010101010101010101010101010101010101";
const STAKING_ADVANCE_ID = "0x0202020202020202020202020202020202020202020202020202020202020202";

var base_fee = 0;

function percentage_of(oldNumber, newNumber) {
    return (1 - (oldNumber / newNumber)) * 100
}

function create_erc20_token_circle(context, name, ticket, expect_rate, tx_limit) {

    step('leave all pool works', async () => {
        console.log("name: ", name);
        const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.leave_all_pool(context, alice);
        console.log("finish");
    }).timeout(10000);

    step(`join pool works`, async () => {
        const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.join_pool(context, alice, ticket);
    }).timeout(10000);

    step(`${name} discount with tx limit works`, async () => {
        let account2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        let count = 0;
        for (let i = 0; i < tx_limit + 2; i++) {
            let before_balance = await context.web3.eth.getBalance(account2.address);
            let receipt = await utils.create_new_contract(context, account2);
            let after_balance = await context.web3.eth.getBalance(account2.address);
            let staking_fee = context.web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
            let rate = percentage_of(staking_fee, base_fee);

            if (count < tx_limit) {
                assert.equal(Math.round(rate), expect_rate);
            } else {
                assert.notEqual(Math.round(rate), expect_rate);
            }
            count++;
        }
    }).timeout(100000);
}


describeWithFrontier("Staking Pool Fee", (context) => {

    step('show total fee spent when ouside the pool', async () => {
        let account = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        let before_balance = await context.web3.eth.getBalance(account.address);

        let receipt = await utils.create_new_contract(context, account);

        let after_balance = await context.web3.eth.getBalance(account.address);
        base_fee = context.web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
        console.log("Base fee: ", base_fee);
    }).timeout(10000);

    step('step should mapping addresses', async () => {
        let account2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.proof_address_mapping(context, account2, alice);
    }).timeout(10000);

    create_erc20_token_circle(context, "Staking", STAKING_BASIC_ID, 10, 100);
    create_erc20_token_circle(context, "Staking", STAKING_MEDIUM_ID, 20, 100);
    create_erc20_token_circle(context, "Staking", STAKING_ADVANCE_ID, 30, 100);
})