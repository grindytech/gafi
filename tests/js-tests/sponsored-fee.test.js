require('dotenv').config();
const Web3 = require('web3');
const web3 = new Web3(process.env.RPC_API);
const chai = require('chai');
chai.use(require('chai-as-promised'));
const { BigNumber } = require('@ethersproject/bignumber');
const utils = require('../utils/utils');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const wsProvider = new WsProvider(process.env.WS_API);
const { Keyring } = require('@polkadot/api');
const keyring = new Keyring({ type: 'sr25519' });
var assert = require('assert');


const alice_pair = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
const bob_pair = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);

var NormalFee;
var ERC20_ADDRESS;
var NewPool;
const DISCOUNT = 19;
const TX_LIMIT = 10;

function delay(interval) {
    return it(`should delay ${interval}`, done => {
        setTimeout(() => done(), interval)
    }).timeout(interval + 100)
}


function percentage_of(oldNumber, newNumber) {
    return (1 - (oldNumber / newNumber)) * 100
}

describe('Contract', () => {

    it('it should create new erc20 token', async () => {
        let before_balance = await web3.eth.getBalance(alice_pair.address);
        let receipt = await utils.create_new_contract(alice_pair);
        ERC20_ADDRESS = receipt.contractAddress;
        let after_balance = await web3.eth.getBalance(alice_pair.address);
    }).timeout(3600000);

    it('it should trasfer erc20 token', async () => {
        let before_balance = await web3.eth.getBalance(alice_pair.address);
        let receipt = await utils.transfer_erc20(ERC20_ADDRESS, alice_pair,
            bob_pair.address, "1000000000000000000");
        let after_balance = await web3.eth.getBalance(alice_pair.address);
        NormalFee = web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
    }).timeout(3600000);

    it('it should mapping addresses', async () => {
        const api = await ApiPromise.create({ provider: wsProvider });
        const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.proof_address_mapping(alice_pair, Alice);
    }).timeout(3600000);

    it('it should mapping addresses', async () => {
        const api = await ApiPromise.create({ provider: wsProvider });
        var Bob = keyring.addFromUri('//Bob', { name: 'Bob default' });

        await utils.proof_address_mapping(bob_pair, Bob);
    }).timeout(3600000);
    delay(6000);

    it('it should create new pool', async () => {
        var Bob = keyring.addFromUri('//Bob', { name: 'Bob default' });

        let value = "1000000000000000000000"; // 1000 GAKI
        let discount = DISCOUNT;
        let txLimit = TX_LIMIT;

        let argument = {
            targets: [ERC20_ADDRESS],
            value: value,
            discount: discount,
            txLimit: txLimit,
        }
        await utils.create_pool(Bob, argument);
    }).timeout(3600000);
    delay(6000);

    it('it should get owned pools before create new pool', async () => {
        const api = await ApiPromise.create({ provider: wsProvider });
        var Bob = keyring.addFromUri('//Bob', { name: 'Bob default' });
        let pools = await api.query.sponsoredPool.poolOwned(Bob.publicKey);
        NewPool = pools[pools.length - 1];
    })

    it('leave pool works', async () => {
        const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.leave_pool(Alice);
    }).timeout(3600000);

    delay(6000);
    it('it should trasfer erc20 token', async () => {
        let before_balance = await web3.eth.getBalance(alice_pair.address);
        let receipt = await utils.transfer_erc20(ERC20_ADDRESS, alice_pair,
            bob_pair.address, "1000000000000000000");
        let after_balance = await web3.eth.getBalance(alice_pair.address);
        NormalFee = web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
    }).timeout(3600000);


    delay(6000);
    it('join sponsored sponsored works', async () => {
        const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.join_pool(Alice, { Sponsored: NewPool });
    }).timeout(3600000);

    delay(6000);
    it('Discount with tx limit works', async () => {
        let count = 0;
        for (let i = 0; i < 11; i++) {
            let before_balance = await web3.eth.getBalance(alice_pair.address);
            await utils.transfer_erc20(ERC20_ADDRESS, alice_pair,
                bob_pair.address, "1000000000000000000");
            let after_balance = await web3.eth.getBalance(alice_pair.address);
            let transfer_fee = web3.utils.fromWei(BigNumber.from(before_balance).sub(BigNumber.from(after_balance)).toString(), "ether");
            let rate = percentage_of(transfer_fee, NormalFee);

            count++;
            if(count <= TX_LIMIT) {
                assert.equal(Math.round(rate), DISCOUNT);
            } else {
                assert.notEqual(Math.round(rate), DISCOUNT);
            }
        }
    }).timeout(3600000);
})
