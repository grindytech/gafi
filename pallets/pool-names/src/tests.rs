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

fn new_sudo_account(balance: u128) -> AccountId32 {
	let ALICE: AccountId32 =
		AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
	make_deposit(&ALICE, balance);
	assert_eq!(Balances::free_balance(&ALICE), balance);
	return ALICE;
}

fn new_account(account: [u8; 32], balance: u128) -> AccountId32 {
    let acc: AccountId32 = AccountId32::from(account);
    make_deposit(&acc, balance);
    assert_eq!(Balances::free_balance(&acc), balance);
    return acc;
}

fn create_pool(
    account: AccountId32,
    targets: Vec<H160>,
    pool_value: u128,
    tx_limit: u32,
    discount: u8,
) -> ID {
    assert_ok!(SponsoredPool::create_pool(
        Origin::signed(account.clone()),
        targets,
        pool_value,
        discount,
        tx_limit
    ));

    let pool_owned = sponsored_pool::Pallet::<Test>::pool_owned(account.clone());

    pool_owned[0]
}

#[test]
fn normal_operation_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let account_balance = 1_001_000 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);
        let pool_value = 1000 * unit(GAKI);
        let pool_id = create_pool(
            account.clone(),
            vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
            pool_value,
            10,
            100,
        );

		assert_ok!(PoolNames::set_name(Origin::signed(account.clone()), pool_id, b"Test pool".to_vec()));
		assert_eq!(Balances::reserved_balance(account.clone()), unit(GAKI));
		assert_eq!(Balances::free_balance(account.clone()), 999_999 * unit(GAKI));
		assert_eq!(<NameOf<Test>>::get(pool_id).unwrap().0, b"Test pool".to_vec());

		assert_ok!(PoolNames::set_name(Origin::signed(account.clone()), pool_id, b"Test pool1".to_vec()));
		assert_eq!(Balances::reserved_balance(account.clone()), unit(GAKI));
		assert_eq!(Balances::free_balance(account.clone()), 999_999 * unit(GAKI));
		assert_eq!(<NameOf<Test>>::get(pool_id).unwrap().0, b"Test pool1".to_vec());

		assert_ok!(PoolNames::clear_name(Origin::signed(account.clone()), pool_id));
		assert_eq!(Balances::reserved_balance(account.clone()), 0);
		assert_eq!(Balances::free_balance(account.clone()), 1_000_000 * unit(GAKI));
	});
}

#[test]
	fn kill_name_should_work() {
		ExtBuilder::default().build_and_execute(|| {
			run_to_block(1);
			let account_balance = 1_001_000 * unit(GAKI);
			let account = new_account([0_u8; 32], account_balance);
			let pool_value = 1000 * unit(GAKI);
			let pool_id = create_pool(
				account.clone(),
				vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
				pool_value,
				10,
				100,
			);
			let sudo_account = new_sudo_account(account_balance);
			assert_ok!(PoolNames::set_name(Origin::signed(account.clone()), pool_id, b"Test pool".to_vec()));
			assert_eq!(Balances::total_balance(&account), 1_000_000 * unit(GAKI));
			assert_ok!(PoolNames::kill_name(Origin::root(), pool_id));
			assert_eq!(Balances::total_balance(&account), 1_000_000 * unit(GAKI) - unit(GAKI));
			assert_eq!(<NameOf<Test>>::get(pool_id), None);
		});
	}

#[test]
fn error_catching_should_work() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let account_balance = 1_001_000 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);
		let account1 = new_account([1_u8; 32], 0);
		let pool_value = 1000 * unit(GAKI);
        let pool_id = create_pool(
            account.clone(),
            vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
            pool_value,
            10,
            100,
        );

		assert_noop!(PoolNames::clear_name(Origin::signed(account.clone()), pool_id), Error::<Test>::Unnamed);

		assert_noop!(
			PoolNames::set_name(Origin::signed(account1.clone()), pool_id, b"Test pool".to_vec()),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);

		assert_noop!(
			PoolNames::set_name(Origin::signed(account.clone()), pool_id, b"Te".to_vec()),
			Error::<Test>::TooShort
		);
		assert_noop!(
			PoolNames::set_name(Origin::signed(account.clone()), pool_id, b"Test pool name with 16 chars".to_vec()),
			Error::<Test>::TooLong
		);
	});
}
