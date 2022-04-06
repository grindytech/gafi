use crate::{mock::*, Error};
use gafi_primitives::{currency::{NativeToken::AUX, unit}};
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use sp_runtime::AccountId32;
use sp_std::str::FromStr;

#[test]
fn stake_pool_works() {
	ExtBuilder::default().build_and_execute(|| {
		let ALICE_BALANCE = 1_000_000_000 * unit(AUX);
		let ALICE: AccountId32 =
			AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
		assert_ok!(StakingPool::stake(Origin::signed(ALICE.clone())));
		assert_eq!(Balances::free_balance(&ALICE), ALICE_BALANCE - STAKE_AMOUNT);
		assert_eq!(Balances::reserved_balance(&ALICE), STAKE_AMOUNT);
	})
}

#[test]
fn stake_pool_fail() {
	ExtBuilder::default().build_and_execute(|| {
		let ALICE: AccountId32 =
			AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		assert_err!(
			StakingPool::stake(Origin::signed(ALICE.clone())),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
		let ALICE_BALANCE = 1_000_000_000 * unit(AUX);
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
		assert_ok!(StakingPool::stake(Origin::signed(ALICE.clone())));
		assert_err!(
			StakingPool::stake(Origin::signed(ALICE.clone())),
			<Error<Test>>::PlayerAlreadyStake
		);
	})
}

#[test]
fn unstake_pool_works() {
	ExtBuilder::default().build_and_execute(|| {
		let ALICE_BALANCE = 1_000_000_000 * unit(AUX);
		let ALICE: AccountId32 =
			AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
		assert_ok!(StakingPool::stake(Origin::signed(ALICE.clone())));
		assert_ok!(StakingPool::unstake(Origin::signed(ALICE.clone())));
		assert_eq!(Balances::free_balance(&ALICE), ALICE_BALANCE);
		assert_eq!(Balances::reserved_balance(&ALICE), 0);
	})
}

#[test]
fn unstake_pool_fail() {
	ExtBuilder::default().build_and_execute(|| {
		let ALICE_BALANCE = 1_000_000_000 * unit(AUX);
		let ALICE: AccountId32 =
			AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
		assert_err!(
			StakingPool::unstake(Origin::signed(ALICE.clone())),
			<Error<Test>>::PlayerNotStake
		);
	})
}

#[test]
fn set_discount_works() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(StakingPool::set_discount(Origin::root(), 2));
	})
}

#[test]
fn set_discount_fail() {
	ExtBuilder::default().build_and_execute(|| {
		let ALICE: AccountId32 =
			AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		// bad origin
		assert_err!(StakingPool::set_discount(Origin::signed(ALICE), 2), frame_support::error::BadOrigin);
		// incorrect discount value
		assert_err!(StakingPool::set_discount(Origin::root(), 101), <Error<Test>>::DiscountNotCorrect);
	})
}
