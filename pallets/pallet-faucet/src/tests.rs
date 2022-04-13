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
