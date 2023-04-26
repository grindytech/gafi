use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::AccountId32;

fn make_deposit(account: &AccountId32, balance: u128) {
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(account: [u8; 32], balance: u128) -> AccountId32 {
	let acc: AccountId32 = AccountId32::from(account);
	make_deposit(&acc, balance);
	assert_eq!(Balances::free_balance(&acc), balance);
	return acc
}


#[test]
fn create_game_should_works() {
	new_test_ext().execute_with(|| {



	});
}

