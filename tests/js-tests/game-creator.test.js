require('dotenv').config();
const Web3 = require('web3');
const chai = require('chai');
chai.use(require('chai-as-promised'));
var assert = require('assert');
const { describeWithFrontier, RPC_PORT } = require('../utils/context');
const util = require('../utils/util');
var ERC20ABI = require('../build/contracts/GAKI.json');


describeWithFrontier("Frontier RPC (EthFilterApi)", (context) => {


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


    it('it should create new erc20 token', async () => {
        const base_account = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
        let before_balance = await context.web3.eth.getBalance(base_account.address);

        let txhash = await util.create_new_contract(context, base_account);
        console.log("txhash: ", txhash);

        let after_balance = await context.web3.eth.getBalance(base_account.address);

    }).timeout(20000);

})



