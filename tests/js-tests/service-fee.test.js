require('dotenv').config();
const Web3 = require('web3');
const web3 = new Web3(process.env.RPC_API);
const chai = require('chai');
chai.use(require('chai-as-promised'));
const { BigNumber } = require('@ethersproject/bignumber');
const utils = require('./utils');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const wsProvider = new WsProvider(process.env.WS_API);
const { Keyring } = require('@polkadot/api');
const keyring = new Keyring({ type: 'sr25519' });
var assert = require('assert');


const test1 = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
const test2 = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
const test3 = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_3);

function delay(interval) {
    return it(`should delay ${interval}`, done => {
        setTimeout(() => done(), interval)
    }).timeout(interval + 100)
}

var nomal_fee;

function percentage_of(oldNumber, newNumber) {
    return (1 - (oldNumber / newNumber)) * 100
}

function join_leave_circle(ticket, expect_rate) {
    delay(7000);
    it('leave pool works', async () => {
        const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.leave_pool(alice);
    }).timeout(3600000);

    delay(7000);
    it('join staking pool basic works', async () => {
        const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.join_pool(alice, ticket);
    }).timeout(3600000);

    delay(7000);
    it('discount of Staking Pool basic should works', async () => {
        let admin = test2;
        let before_balance = await web3.eth.getBalance(admin.address);
        console.log("deploying...");
        let receipt = await utils.create_new_contract(admin);
        let after_balance = await web3.eth.getBalance(admin.address);
        let staking_fee = web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
        let rate = percentage_of(staking_fee, nomal_fee);
        assert.equal(Math.round(rate), expect_rate);
    }).timeout(3600000);
}

describe('Contract', () => {

    it('show total fee spent when ouside the pool', async () => {
        let admin = test1;
        let before_balance = await web3.eth.getBalance(admin.address);
        console.log("deploying...");
        let receipt = await utils.create_new_contract(admin);
        let after_balance = await web3.eth.getBalance(admin.address);
        nomal_fee = web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
        console.log("Normal fee: ", nomal_fee);
    }).timeout(3600000);

    it('it should mapping addresses', async () => {
        const api = await ApiPromise.create({ provider: wsProvider });
        let admin = test2;
        const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.proof_address_mapping(admin, alice);
    }).timeout(3600000);

    join_leave_circle({ Staking: "Basic" }, 30);
    join_leave_circle({ Staking: "Medium" }, 50);
    join_leave_circle({ Staking: "Advance" }, 70);


    join_leave_circle({ Upfront: "Basic" }, 30);
    join_leave_circle({ Upfront: "Medium" }, 50);
    join_leave_circle({ Upfront: "Advance" }, 70);
})

