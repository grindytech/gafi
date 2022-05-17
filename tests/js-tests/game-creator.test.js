require('dotenv').config();
const Web3 = require('web3');
const chai = require('chai');
chai.use(require('chai-as-promised'));
var assert = require('assert');
const {describeWithFrontier, RPC_PORT} = require('../utils/context');

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

    describe('Contract', () => {
    
        it('it should create new erc20 token', async () => {
            console.log(context);
            const alice_pair = context.web3.eth.accounts.privateKeyToAccount("0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342");
            
            let before_balance = await context.web3.eth.getBalance(alice_pair.address);
            let receipt = await utils.create_new_contract(alice_pair);
            ERC20_ADDRESS = receipt.contractAddress;
            let after_balance = await context.web3.eth.getBalance(alice_pair.address);
    
            console.log("ERC20_ADDRESS: ", ERC20_ADDRESS);
        }).timeout(3600000);
    })

})



