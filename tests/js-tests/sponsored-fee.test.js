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

var NormalFee;
var ERC20_ADDRESS;
var NewPool;
const DISCOUNT = 50;
const TX_LIMIT = 10;

function delay(interval) {
    return it(`should delay ${interval}`, done => {
        setTimeout(() => done(), interval)
    }).timeout(interval + 100)
}

function percentage_of(oldNumber, newNumber) {
    return (1 - (Number(oldNumber) / Number(newNumber))) * 100;
}

/// Situation: Alice map with account_1, Bob map with account_2
/// Alice is the pool owner, Bob is the player
describeWithFrontier("Upfront and Staking Pool Fee", (context) => {

    it('it should create new erc20 token', async () => {
        const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        let receipt = await utils.create_new_contract(context, account_1);
        ERC20_ADDRESS = receipt.contractAddress;
    }).timeout(20000);

    it('it should trasfer erc20 token', async () => {
        const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        const account_2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        let token_balance = "10000000000000";
        await utils.transfer_erc20(context, ERC20_ADDRESS, account_1, account_2.address, token_balance);
        let balance = await utils.get_erc20_balance(context, ERC20_ADDRESS, account_2.address);
        assert.deepEqual(balance.toString(), token_balance, "transfer balance not correct");
    }).timeout(20000);

    it('it should trasfer erc20 token to get normal fee', async () => {
        const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        const account_2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        let token_balance = "10000000000000";
        let before_balance = await context.web3.eth.getBalance(account_1.address);
        await utils.transfer_erc20(context, ERC20_ADDRESS, account_1, account_2.address, token_balance);
        let after_balance = await context.web3.eth.getBalance(account_1.address);
        NormalFee = context.web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
        console.log(`NormalFee: ${NormalFee}`);
    }).timeout(20000);

    it('it should mapping addresses', async () => {
        const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        const wsProvider = new WsProvider(`ws://127.0.0.1:${WS_PORT}`);
        const api = await ApiPromise.create({ provider: wsProvider });
        const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.proof_address_mapping(context, account_1, Alice);
    }).timeout(20000);

    it('it should mapping addresses', async () => {
        const account_2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        const wsProvider = new WsProvider(`ws://127.0.0.1:${WS_PORT}`);
        const api = await ApiPromise.create({ provider: wsProvider });
        var Bob = keyring.addFromUri('//Bob', { name: 'Bob default' });
        await utils.proof_address_mapping(context, account_2, Bob);
    }).timeout(20000);

    it('it should create new pool', async () => {
        const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        let value = "1000000000000000000000"; // 1000 GAKI
        let discount = DISCOUNT;
        let txLimit = TX_LIMIT;

        let argument = {
            targets: [ERC20_ADDRESS],
            value: value,
            discount: discount,
            txLimit: txLimit,
        }
        await utils.create_pool(context, Alice, argument);
    }).timeout(20000);

    it('it should get owned pools before create new pool', async () => {
        const wsProvider = new WsProvider(`ws://127.0.0.1:${WS_PORT}`);
        const api = await ApiPromise.create({ provider: wsProvider });
        const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        let pools = await api.query.sponsoredPool.poolOwned(Alice.publicKey);
        NewPool = pools[pools.length - 1];
    })

    it('leave any pool before join sponsored pool works', async () => {
        var Bob = keyring.addFromUri('//Bob', { name: 'Bob default' });
        await utils.leave_pool(context, Bob);
    }).timeout(20000);


    it('join sponsored sponsored works', async () => {
        var Bob = keyring.addFromUri('//Bob', { name: 'Bob default' });
        await utils.join_pool(context, Bob, { Sponsored: NewPool });
    }).timeout(20000);

    it('discount on sponsored pool works', async () => {
        const wsProvider = new WsProvider(`ws://127.0.0.1:${WS_PORT}`);
        const api = await ApiPromise.create({ provider: wsProvider });
        const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        const account_2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        let before_balance = await context.web3.eth.getBalance(account_2.address);

        let account = await api.query.system.account(NewPool);
        let pool_before_balance = BigNumber.from(account.data.free.toString()).toString();
        console.log(`pool_before_balance: ${pool_before_balance} GAKI`);

        let token_balance = "10000000000000";
        let receipt = await utils.transfer_erc20(context, ERC20_ADDRESS, account_2,
            account_1.address, token_balance);

        let after_balance = await context.web3.eth.getBalance(account_2.address);
        let pool_after_balance = BigNumber.from(account.data.free.toString()).toString();
        console.log(`pool_before_balance: ${pool_before_balance} GAKI`);

        let discount_fee = context.web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
        let pool_fee = context.web3.utils.fromWei(BigNumber.from(pool_before_balance).sub(BigNumber.from(pool_after_balance)).toString(), "ether");
        console.log(`discount_fee: ${discount_fee} GAKI`);
        console.log(`pool_fee: ${pool_fee} GAKI`);
        let rate = percentage_of(discount_fee, NormalFee);
        assert.equal(Math.round(rate), DISCOUNT);
    })

    // delay(6000);
    // it('Discount with tx limit works', async () => {
    //     let count = 0;
    //     for (let i = 0; i < 11; i++) {
    //         let before_balance = await web3.eth.getBalance(account_1.address);
    //         await utils.transfer_erc20(ERC20_ADDRESS, account_1,
    //             account_2.address, "1000000000000000000");
    //         let after_balance = await web3.eth.getBalance(account_1.address);
    //         let transfer_fee = web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
    //         let rate = percentage_of(transfer_fee, NormalFee);

    //         count++;
    //         if(count <= TX_LIMIT) {
    //             assert.equal(Math.round(rate), DISCOUNT);
    //         } else {
    //             assert.notEqual(Math.round(rate), DISCOUNT);
    //         }
    //     }
    // }).timeout(20000);
})
