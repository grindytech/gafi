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


const alice_pair = web3.eth.accounts.privateKeyToAccount("0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342");

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

        console.log("ERC20_ADDRESS: ", ERC20_ADDRESS);
    }).timeout(3600000);
})