use crate::{mock::*, Error};
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	system_services::SystemPool,
};

#[test]
fn membership_registration_should_work() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, 1_000_000 * unit(GAKI));
		assert_ok!(GafiMembership::registration(Origin::signed(ALICE)));

		run_to_block(20);

		assert_eq!(GafiMembership::members(&ALICE).unwrap().is_reached, false);
	});
}

#[test]
fn membership_hook_check_user_reached_should_work() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, 1_000_000 * unit(GAKI));
		assert_ok!(GafiMembership::registration(Origin::signed(ALICE)));
		run_to_block(20);
		let _result = upfront_pool::Pallet::<Test>::join(ALICE, UPFRONT_BASIC_ID);
		run_to_block(2000);

		assert_eq!(GafiMembership::members(&ALICE).unwrap().is_reached, true);
	});
}

#[test]
fn membership_registration_should_dispatch_error_when_already_registered() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, 1_000_000 * unit(GAKI));
		assert_ok!(GafiMembership::registration(Origin::signed(ALICE)));

		run_to_block(20);

		assert_err!(
			GafiMembership::registration(Origin::signed(ALICE)),
			<Error<Test>>::AlreadyRegistered
		);
	});
}
