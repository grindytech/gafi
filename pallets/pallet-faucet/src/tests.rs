use std::ops::Add;

use crate::{mock::*, Error, GenesisAccounts};
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use sp_runtime::{traits::BadOrigin, AccountId32};

#[test]
fn faucet_works() {
	ExtBuilder::default().build_and_execute(|| {
		let sender = AccountId32::new([11; 32]);
		assert_eq!(Balances::free_balance(&sender), 0);
		assert_ok!(Faucet::faucet(RuntimeOrigin::signed(sender.clone())));
		assert_eq!(Balances::free_balance(&sender), FAUCET_BALANCE);
	})
}

#[test]
fn faucet_works_with_low_balance() {
	ExtBuilder::default().build_and_execute(|| {
		let sender = AccountId32::new([11; 32]);
		let legit_balance = FAUCET_BALANCE / 10 - 1u128;
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&sender, legit_balance);
		assert_eq!(Balances::free_balance(&sender), legit_balance);
		assert_ok!(Faucet::faucet(RuntimeOrigin::signed(sender.clone())));
		assert_eq!(
			Balances::free_balance(&sender),
			FAUCET_BALANCE + legit_balance
		);
	})
}

#[test]
fn faucet_fail() {
	ExtBuilder::default().build_and_execute(|| {
		let sender = AccountId32::new([11; 32]);
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&sender, FAUCET_BALANCE);
		assert_err!(
			Faucet::faucet(RuntimeOrigin::signed(sender.clone())),
			<Error<Test>>::DontBeGreedy
		);
	})
}

#[test]
fn faucet_should_fail_when_still_in_cache_time() {
	ExtBuilder::default().build_and_execute(|| {
		let sender = AccountId32::new([11; 32]);
		assert_ok!(Faucet::faucet(RuntimeOrigin::signed(sender.clone())));
		assert_err!(
			Faucet::faucet(RuntimeOrigin::signed(sender.clone())),
			<Error<Test>>::PleaseWait
		);
	})
}

#[test]
fn donate_work() {
	ExtBuilder::default().build_and_execute(|| {
		let sender = AccountId32::new([11; 32]);
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&sender, FAUCET_BALANCE);

		let before_balance = Balances::free_balance(GENESIS_ACCOUNT.clone());

		let _ = Faucet::donate(RuntimeOrigin::signed(sender.clone()), 400_000);

		assert_eq!(
			Balances::free_balance(GENESIS_ACCOUNT.clone()),
			before_balance.add(400_000)
		)
	})
}

#[test]
fn donate_fail() {
	ExtBuilder::default().build_and_execute(|| {
		let sender = AccountId32::new([11; 32]);
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&sender, FAUCET_BALANCE);

		assert_err!(
			Faucet::donate(
				RuntimeOrigin::signed(sender.clone()),
				FAUCET_BALANCE.add(100_000)
			),
			<Error<Test>>::NotEnoughBalance
		);
	})
}

#[test]
fn test_new_funding_accounts() {
	ExtBuilder::default().build_and_execute(|| {
		// Set up test environment
		let root = AccountId32::new([0; 32]);
		let accounts: Vec<AccountId32> = vec![
			AccountId32::new([0; 32]),
			AccountId32::new([1; 32]),
			AccountId32::new([2; 32]),
		];

		// Ensure the function fails when called by a non-root account
		assert_noop!(
			Faucet::new_funding_accounts(RuntimeOrigin::signed(root.clone()), accounts.clone()),
			BadOrigin
		);

		// Ensure the function succeeds when called by a root account
		assert_ok!(Faucet::new_funding_accounts(
			RuntimeOrigin::root(),
			accounts.clone()
		));

		// Verify that the funding accounts were stored correctly
		assert_eq!(GenesisAccounts::<Test>::get(), accounts);
	});
}
