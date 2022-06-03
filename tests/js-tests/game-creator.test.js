require('dotenv').config();
const Web3 = require('web3');
const chai = require('chai');
chai.use(require('chai-as-promised'));
const { step } = require("mocha-steps");
var assert = require('assert');
const { describeWithFrontier, RPC_PORT, createAndFinalizeBlock, describe_with_frontier, WS_PORT} = require('../utils/context');
const util = require('../utils/util');
const { Keyring, WsProvider } = require('@polkadot/api');
const { BigNumber } = require('@ethersproject/bignumber');
const keyring = new Keyring({ type: 'sr25519' });


describeWithFrontier("Frontier RPC (EthFilterApi)", (context) => {
  let transaction_fee;
  var ERC20_ADDRESS;
  let before_receive_reward;
  const DISCOUNT = 190000;
  const TX_LIMIT = 10;
  let token_balance = "20000000000000";
  const token_balance1 = "10000000000000";
  let wsProvider;

  beforeEach("Start ", () => {
    wsProvider = new WsProvider(`ws://127.0.0.1:${WS_PORT}`);
  })

  afterEach("Close", async () => {
    await wsProvider.disconnect()
  })

  function delay(interval) {
      return it(`should delay ${interval}`, done => {
          setTimeout(() => done(), interval)
      }).timeout(interval + 100)
  }

  function percentage_of(oldNumber, newNumber) {
      return (1 - (oldNumber / newNumber)) * 100
  }

  step('it should create new erc20 token', async () => {
    const base_account = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
      let before_balance = await context.web3.eth.getBalance(base_account.address);
      console.log("before_balance: ", before_balance);

      let result = await util.create_new_contract(context, base_account);
      ERC20_ADDRESS = result.contractAddress
      console.log("new token: ", result.contractAddress);

      let after_balance = await context.web3.eth.getBalance(base_account.address);
      console.log("after_balance: ", after_balance);

  }).timeout(20000);

  step('step should transfer erc20 token to get base amount of money', async () => {
    const base_account = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
    const account_2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);

    await util.transfer_erc20(context, ERC20_ADDRESS, base_account, account_2.address, token_balance);

    let account2_balance = await context.web3.eth.getBalance(account_2.address);
    console.log('account2_balance', account2_balance)
  });

  step('it should claim contract success', async () => {
    const base_account = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
    const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });

    await util.proof_address_mapping(context, wsProvider, base_account, alice);
    await util.claim_contract(context, wsProvider, alice, { contractAddress: ERC20_ADDRESS });

    before_receive_reward = await context.web3.eth.getBalance(base_account.address);
    console.log('before_receive_reward', before_receive_reward)
  });

  step('step should transfer erc20 token to get transaction fee', async () => {
    const account_2 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_2);
    const account_3 = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_3);

    let normal_before_balance = await context.web3.eth.getBalance(account_2.address);

    await util.transfer_erc20(context, ERC20_ADDRESS,  account_2, account_3.address, token_balance1);

    let normal_after_balance = await context.web3.eth.getBalance(account_2.address);

    transaction_fee = context.web3.utils.fromWei(BigNumber.from(normal_before_balance).sub(BigNumber.from(normal_after_balance)).sub(BigNumber.from(token_balance1)).toString(), "ether");
    console.log('transaction_fee', transaction_fee)
  });

  step('it should transfer erc20 token to get claimed amount', async () => {
    const base_account = context.web3.eth.accounts.privateKeyToAccount(process.env.PRI_KEY_1);
    let after_receive_reward = await context.web3.eth.getBalance(base_account.address);
    console.log("after_receive_reward: ", after_receive_reward);

    const reward_percent = await util.get_game_creator_reward(wsProvider);
    console.log('rewardPercent', reward_percent.toNumber());

    let received_reward = context.web3.utils.fromWei(BigNumber.from(after_receive_reward).sub(BigNumber.from(before_receive_reward)).toString(), "ether");
    console.log('received_reward', received_reward)

    const percent = Math.round(received_reward / transaction_fee * 100);
    console.log('percent', percent)

    assert.equal(percent, reward_percent, "game creator reward percent not correct");
  }).timeout(20000);
})
