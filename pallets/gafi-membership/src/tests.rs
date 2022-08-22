use crate::{mock::*, Error};
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	membership::{Achievement, Achievements},
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
fn membership_hook_add_point_for_user_should_work() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, 1_000_000 * unit(GAKI));
		assert_ok!(GafiMembership::registration(Origin::signed(ALICE)));
		run_to_block(20);
		let _result = upfront_pool::Pallet::<Test>::join(ALICE, UPFRONT_BASIC_ID);
		run_to_block(2000);

		assert_eq!(
			GafiMembership::members(&ALICE).unwrap().membership_point,
			MembershipAchievements::get_membership_achievements()[0].get_achievement_point()
		);
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

#[test]
fn membership_registration_should_dispatch_error_when_exceed_max_members() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		let accounts = vec![[0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32]];

		for account in accounts {
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(
				&AccountId32::new(account),
				one_mil_gaki(),
			);
			if account == [5; 32] {
				assert_err!(
					GafiMembership::registration(Origin::signed(AccountId32::new(account))),
					<Error<Test>>::ExceedMaxMembers
				);
			} else {
				assert_ok!(GafiMembership::registration(Origin::signed(
					AccountId32::new(account)
				)));
			}
		}

		run_to_block(20);
	});
}
