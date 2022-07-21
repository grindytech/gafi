use crate::{mock::*, Error, Config};
use frame_support::{assert_err, assert_ok, traits::Currency};

// TODO: Add more test later.
#[test]
fn membership_registration_should_work() {
	new_test_ext().execute_with(|| {
		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, 1_000_000);

		assert_ok!(GafiMembership::registration(Origin::signed(ALICE)));
	});
}
