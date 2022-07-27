use crate::{mock::*, Error};
use codec::Encode;
use frame_support::{assert_ok, traits::Currency, assert_err};
use gafi_primitives::{ticket::{TicketLevel, SystemTicket}, system_services::SystemPool, currency::{unit, NativeToken::GAKI}};
use sp_io::hashing::blake2_256;

#[test]
fn membership_registration_should_work() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, 1_000_000 * unit(GAKI));

		let _result = upfront_pool::Pallet::<Test>::join(ALICE, (SystemTicket::Upfront(TicketLevel::Basic)).using_encoded(blake2_256));

		run_to_block(2000);

		assert_ok!(GafiMembership::registration(Origin::signed(ALICE)));
	});
}

#[test]
fn membership_registration_should_get_error_not_eligible() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, 1_000_000 * unit(GAKI));

		let _result = upfront_pool::Pallet::<Test>::join(ALICE, (SystemTicket::Upfront(TicketLevel::Basic)).using_encoded(blake2_256));

		run_to_block(20);

		assert_err!(GafiMembership::registration(Origin::signed(ALICE)), <Error<Test>>::NotEligibleForRegistration);
	});
}
