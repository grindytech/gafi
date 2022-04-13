use std::{ops::Add};

use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, traits::Currency, assert_err};
use sp_runtime::AccountId32;

#[test]
fn faucet_works() {
	ExtBuilder::default().build_and_execute(|| {
		let sender = AccountId32::new([11; 32]);
		assert_eq!(Balances::free_balance(&sender), 0);
		assert_ok!(Faucet::faucet(Origin::signed(sender.clone())));
		assert_eq!(Balances::free_balance(&sender), FAUCET_BALANCE);
	})
}

#[test]
fn faucet_fail() {
	ExtBuilder::default().build_and_execute(|| {
		let sender = AccountId32::new([11;32]);
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&sender, MIN_FAUCET_BALANCE);
		assert_err!(Faucet::faucet(Origin::signed(sender.clone())), <Error<Test>>::DontBeGreedy);
	})
}

#[test]
fn donate_work() {
	ExtBuilder::default().build_and_execute(|| {
		let sender = AccountId32::new([11;32]);
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&sender, MIN_FAUCET_BALANCE);

		let before_balance = Balances::free_balance(GENESIS_ACCOUNT.clone());

		let _ = Faucet::donate(Origin::signed(sender.clone()), 400_000);

		assert_eq!(Balances::free_balance(GENESIS_ACCOUNT.clone()), before_balance.add(400_000))
	})
}

#[test]
fn donate_fail() {
	ExtBuilder::default().build_and_execute(|| {
		let sender = AccountId32::new([11;32]);
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&sender, MIN_FAUCET_BALANCE);

		assert_err!(Faucet::donate(Origin::signed(sender.clone()), MIN_FAUCET_BALANCE.add(100_000)), <Error<Test>>::NotEnoughBalance);
	})
}
