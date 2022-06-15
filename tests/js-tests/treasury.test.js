require('dotenv').config();
const Web3 = require('web3');
const chai = require('chai');
chai.use(require('chai-as-promised'));
const { BigNumber } = require('@ethersproject/bignumber');
const utils = require('../utils/util');
const { Keyring } = require('@polkadot/api');
const keyring = new Keyring({ type: 'sr25519' });
var assert = require('assert');
const { describeWithFrontier, WS_PORT, RPC_PORT, createAndFinalizeBlock } = require('../utils/context');
const { step } = require("mocha-steps");

const BASE_TREASURY_BALANCE = 1 * 10 ** 18;

/// Situation: Alice is making transactions.
/// A part of the transactions fee is transferred to treasury.
describeWithFrontier("Treasury", (context) => {

  step('Should create new erc20 token', async () => {
      const account_1 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
      let receipt = await utils.create_new_contract(context, account_1);
      ERC20_ADDRESS = receipt.contractAddress;
  }).timeout(20000);

  step('Should increasing treasury amount', async () => {
    const treasury_balance = await utils.get_treasury_balance(context.provider);

    chai.expect(treasury_balance.freeBalance.gt(BASE_TREASURY_BALANCE)).to.be.true;
  }).timeout(20000);

})
