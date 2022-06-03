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
const { describeWithFrontier, WS_PORT, RPC_PORT, createAndFinalizeBlock } = require('../utils/context');
const { step } = require("mocha-steps");

var NormalFee;
var ERC20_ADDRESS;
var NewPool;
const DISCOUNT = 60;
const TX_LIMIT = 50;

function delay(interval) {
    return step(`should delay ${interval}`, done => {
        setTimeout(() => done(), interval)
    }).timeout(interval + 100)
}

function percentage_of(oldNumber, newNumber) {
    return (1 - (Number(oldNumber) / Number(newNumber))) * 100;
}

/// Situation: Alice map with account_1, Bob map with account_2
/// Alice is the pool owner, Bob is the player
describeWithFrontier("Upfront and Staking Pool Fee", (context) => {
    let wsProvider;

    beforeEach("Start ", () => {
      wsProvider = new WsProvider(`ws://127.0.0.1:${WS_PORT}`);
    })

    afterEach("Close", async () => {
      await wsProvider.disconnect()
    })

    step('step should create new erc20 token', async () => {
        const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        let receipt = await utils.create_new_contract(context, account_1);
        ERC20_ADDRESS = receipt.contractAddress;
    }).timeout(20000);

    step('step should trasfer erc20 token', async () => {
        const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        const account_2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        let token_balance = "10000000000000";
        await utils.transfer_erc20(context, ERC20_ADDRESS, account_1, account_2.address, token_balance);
        let balance = await utils.get_erc20_balance(context, ERC20_ADDRESS, account_2.address);
        assert.deepEqual(balance.toString(), token_balance, "transfer balance not correct");
    }).timeout(20000);

    step('step should trasfer erc20 token to get normal fee', async () => {
        const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        const account_2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        let token_balance = "10000000000000";
        let before_balance = await context.web3.eth.getBalance(account_1.address);
        await utils.transfer_erc20(context, ERC20_ADDRESS, account_1, account_2.address, token_balance);
        let after_balance = await context.web3.eth.getBalance(account_1.address);
        NormalFee = context.web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
        console.log(`NormalFee: ${NormalFee}`);
    }).timeout(20000);

    step('step should mapping addresses', async () => {
        const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);

        const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.proof_address_mapping(context, wsProvider, account_1, Alice);
    }).timeout(20000);

    step('step should mapping addresses', async () => {
        const account_2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        var Bob = keyring.addFromUri('//Bob', { name: 'Bob default' });
        await utils.proof_address_mapping(context, wsProvider, account_2, Bob);
    }).timeout(20000);

    step('step should create new pool', async () => {
        const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        let value = "1000000000000000000000"; // 1000 GAKI
        let discount = DISCOUNT * 10000;
        let txLimit = TX_LIMIT;

        let argument = {
            targets: [ERC20_ADDRESS],
            value: value,
            discount: discount,
            txLimit: txLimit,
        }
        await utils.create_pool(context, wsProvider, Alice, argument);
    }).timeout(20000);

    step('step should get owned pools before create new pool', async () => {
        const api = await ApiPromise.create({ provider: wsProvider });
        const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        let pools = await api.query.sponsoredPool.poolOwned(Alice.publicKey);
        NewPool = pools[pools.length - 1];
    })

    step('leave any pool before join sponsored pool works', async () => {
        var Bob = keyring.addFromUri('//Bob', { name: 'Bob default' });
        await utils.leave_pool(context, wsProvider, Bob);
    }).timeout(20000);


    step('join sponsored sponsored works', async () => {
        var Bob = keyring.addFromUri('//Bob', { name: 'Bob default' });
        await utils.join_pool(context, wsProvider, Bob, { Custom: { Sponsored: NewPool } });
    }).timeout(20000);

    step('discount on sponsored pool works', async () => {
        const api = await ApiPromise.create({ provider: wsProvider });
        const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        const account_2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
        let before_balance = await context.web3.eth.getBalance(account_2.address);

        let pool_account_before = await api.query.system.account(NewPool);
        let pool_before_balance = BigNumber.from(pool_account_before.data.free.toString()).toString();

        let token_balance = "10000000000";
        await utils.transfer_erc20(context, ERC20_ADDRESS, account_2, account_1.address, token_balance);

        let after_balance = await context.web3.eth.getBalance(account_2.address);
        let pool_account_after = await api.query.system.account(NewPool);
        let pool_after_balance = BigNumber.from(pool_account_after.data.free.toString()).toString();

        let discount_fee = context.web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
        let pool_fee = context.web3.utils.fromWei(BigNumber.from(pool_before_balance).sub(BigNumber.from(pool_after_balance)).toString(), "ether");

        let discount_rate = percentage_of(discount_fee, NormalFee);
        assert.equal(Math.round(discount_rate), DISCOUNT);

        let pool_fee_rate = percentage_of(pool_fee, NormalFee);
        assert.equal(Math.round(pool_fee_rate), 100 - DISCOUNT);

    }).timeout(20000);

    step('Discount with tx limit works', async () => {
        const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        const account_2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);

        let count = 0;
        for (let i = 0; i < TX_LIMIT + 2; i++) {
            let before_balance = await context.web3.eth.getBalance(account_2.address);
            await utils.transfer_erc20(context, ERC20_ADDRESS, account_2,
                account_1.address, "100");
            let after_balance = await context.web3.eth.getBalance(account_2.address);
            let transfer_fee = context.web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
            let rate = percentage_of(transfer_fee, NormalFee);

            count++;
            if (count <= TX_LIMIT - 1) {
                assert.equal(Math.round(rate), DISCOUNT);
            } else {
                assert.notEqual(Math.round(rate), DISCOUNT);
            }
        }
    }).timeout(30000);
})
