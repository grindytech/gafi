use core::{str::FromStr};

use super::*;
use crate::mocks::*;

use frame_support::{
	assert_noop, assert_ok,
};
use sp_core::H160;
use sp_runtime::AccountId32;
use gafi_primitives::currency::{unit, NativeToken::GAKI};

fn make_deposit(account: &AccountId32, balance: u128) {
    let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(account: [u8; 32], balance: u128) -> AccountId32 {
    let acc: AccountId32 = AccountId32::from(account);
    make_deposit(&acc, balance);
    assert_eq!(Balances::free_balance(&acc), balance);
    return acc;
}

#[test]
fn normal_operation_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);
        let asset_id = [0_u8; 32];

		let free_balance =account_balance - RESERVATION_FEE * unit(GAKI);

		assert_ok!(PoolNames::set_name(account.clone(), asset_id, b"Test pool".to_vec()));
		assert_eq!(Balances::reserved_balance(account.clone()), RESERVATION_FEE * unit(GAKI));
		assert_eq!(Balances::free_balance(account.clone()),free_balance );
		assert_eq!(<NameOf<Test>>::get(asset_id).unwrap().0, b"Test pool".to_vec());

		assert_ok!(PoolNames::set_name(account.clone(), asset_id, b"Test pool1".to_vec()));
		assert_eq!(Balances::reserved_balance(account.clone()), RESERVATION_FEE * unit(GAKI));
		assert_eq!(Balances::free_balance(account.clone()), free_balance);
		assert_eq!(<NameOf<Test>>::get(asset_id).unwrap().0, b"Test pool1".to_vec());

		assert_ok!(PoolNames::clear_name(account.clone(), asset_id));
		assert_eq!(Balances::reserved_balance(account.clone()), 0);
		assert_eq!(Balances::free_balance(account.clone()), account_balance);
	});
}

#[test]
fn kill_name_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
		let account = new_account([0_u8; 32], account_balance);
		let asset_id = [0_u8; 32];

		assert_ok!(PoolNames::set_name(account.clone(), asset_id, b"Test pool".to_vec()));
		assert_eq!(Balances::total_balance(&account), account_balance);
		assert_ok!(PoolNames::kill_name(account.clone(), asset_id));
		assert_eq!(Balances::total_balance(&account), account_balance - RESERVATION_FEE * unit(GAKI));
		assert_eq!(<NameOf<Test>>::get(asset_id), None);
	});
}

#[test]
fn error_catching_should_work() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);
		let account1 = new_account([1_u8; 32], 0);
        let asset_id = [0_u8; 32];

		assert_noop!(PoolNames::clear_name(account.clone(), asset_id), Error::<Test>::Unnamed);

		assert_noop!(
			PoolNames::set_name(account1.clone(), asset_id, b"Test pool".to_vec()),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);

		assert_noop!(
			PoolNames::set_name(account.clone(), asset_id, b"Te".to_vec()),
			Error::<Test>::TooShort
		);
		assert_noop!(
			PoolNames::set_name(account.clone(), asset_id, b"Test pool name with 16 chars".to_vec()),
			Error::<Test>::TooLong
		);
	});
}
