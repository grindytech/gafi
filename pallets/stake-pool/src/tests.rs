use crate::{mock::*, Error};
use aurora_primitives::{unit, currency::NativeToken::AUX};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::AccountId32;
use sp_std::{str::FromStr};



#[test]
fn stake_pool_works() {
	ExtBuilder::default().build_and_execute(|| {
		let ALICE_BALANCE = 1_000_000_000 * unit(AUX);
		let ALICE: AccountId32 = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
		assert_ok!(StakePool::stake(Origin::signed(ALICE.clone())));
		assert_eq!(Balances::free_balance(&ALICE), ALICE_BALANCE - STAKE_AMOUNT );
		assert_eq!(Balances::reserved_balance(&ALICE), STAKE_AMOUNT );
	})
}

