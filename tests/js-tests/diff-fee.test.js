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


const test1 = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
const test2 = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
const test3 = web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_3);

const TEST_COUNT = process.env.TEST_COUNT;

function delay(interval) {
    return it(`should delay ${interval}`, done => {
        setTimeout(() => done(), interval)
    }).timeout(interval + 100)
}

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

    it('it should mapping addresses', async () => {
        const api = await ApiPromise.create({ provider: wsProvider });
        let admin = test2;
        const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.proof_address_mapping(admin, alice);
    }).timeout(3600000);

    delay(10000);

    it('it should join the pool', async () => {
        const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
        await utils.join_pool(alice, "Staking");
    }).timeout(3600000);

    delay(10000);

    it('show total fee spent when inside the pool', async () => {
        let admin = test2;

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
})

