use core::str::FromStr;

use super::*;
use crate::{mocks::*,};

use frame_support::{
	assert_noop, assert_ok,
};
use sp_core::H160;
use sp_runtime::AccountId32;
use sp_runtime::{
	traits::{BadOrigin},
};
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

fn create_pool(
    account: AccountId32,
    account_balance: u128,
    targets: Vec<H160>,
    pool_value: u128,
    tx_limit: u32,
    discount: u8,
) -> ID {
    SponsoredPool::create_pool(
        Origin::signed(account.clone()),
        targets,
        pool_value,
        discount,
        tx_limit
    );

    let pool_owned = sponsored_pool::Pallet::<Test>::pool_owned(account.clone());
    // assert_eq!(pool_owned.len(), 1);
    // let new_pool = Pools::get(pool_owned[0]).unwrap();
    // assert_eq!(new_pool.owner, account);
    // assert_eq!(new_pool.tx_limit, tx_limit);
    // assert_eq!(new_pool.discount, discount);
    pool_owned[0]
}

#[test]
// fn kill_name_should_work() {
// 	new_test_ext().execute_with(|| {
// 		assert_ok!(PoolNames::set_name(Origin::signed(2), , b"Dave".to_vec()));
// 		assert_eq!(Balances::total_balance(&2), 10);
// 		assert_ok!(Nicks::kill_name(Origin::signed(1), 2));
// 		assert_eq!(Balances::total_balance(&2), 8);
// 		assert_eq!(<NameOf<Test>>::get(2), None);
// 	});
// }

// #[test]
// fn force_name_should_work() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(
// 			Nicks::set_name(Origin::signed(2), b"Dr. David Brubeck, III".to_vec()),
// 			Error::<Test>::TooLong,
// 		);

// 		assert_ok!(Nicks::set_name(Origin::signed(2), b"Dave".to_vec()));
// 		assert_eq!(Balances::reserved_balance(2), 2);
// 		assert_noop!(
// 			Nicks::force_name(Origin::signed(1), 2, b"Dr. David Brubeck, III".to_vec()),
// 			Error::<Test>::TooLong,
// 		);
// 		assert_ok!(Nicks::force_name(Origin::signed(1), 2, b"Dr. Brubeck, III".to_vec()));
// 		assert_eq!(Balances::reserved_balance(2), 2);
// 		let (name, amount) = <NameOf<Test>>::get(2).unwrap();
// 		assert_eq!(name, b"Dr. Brubeck, III".to_vec());
// 		assert_eq!(amount, 2);
// 	});
// }

#[test]
fn normal_operation_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		let account_balance = 1_000_000 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);
        let pool_value = 1000 * unit(GAKI);
        let pool_id = create_pool(
            account,
            account_balance,
            vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
            pool_value,
            10,
            100,
        );

		assert_ok!(PoolNames::set_name(Origin::signed(account.clone()), pool_id, b"Test pool".to_vec()));
		// assert_eq!(Balances::reserved_balance(1), 2);
		// assert_eq!(Balances::free_balance(1), 8);
		assert_eq!(<NameOf<Test>>::get(account.clone()).unwrap().0, b"Test pool".to_vec());

		// assert_ok!(Nicks::set_name(Origin::signed(1), b"Gavin".to_vec()));
		// assert_eq!(Balances::reserved_balance(1), 2);
		// assert_eq!(Balances::free_balance(1), 8);
		// assert_eq!(<NameOf<Test>>::get(1).unwrap().0, b"Gavin".to_vec());

		// assert_ok!(Nicks::clear_name(Origin::signed(1)));
		// assert_eq!(Balances::reserved_balance(1), 0);
		// assert_eq!(Balances::free_balance(1), 10);
	});
}

// #[test]
// fn error_catching_should_work() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(Nicks::clear_name(Origin::signed(1)), Error::<Test>::Unnamed);

// 		assert_noop!(
// 			Nicks::set_name(Origin::signed(3), b"Dave".to_vec()),
// 			pallet_balances::Error::<Test, _>::InsufficientBalance
// 		);

// 		assert_noop!(
// 			Nicks::set_name(Origin::signed(1), b"Ga".to_vec()),
// 			Error::<Test>::TooShort
// 		);
// 		assert_noop!(
// 			Nicks::set_name(Origin::signed(1), b"Gavin James Wood, Esquire".to_vec()),
// 			Error::<Test>::TooLong
// 		);
// 		assert_ok!(Nicks::set_name(Origin::signed(1), b"Dave".to_vec()));
// 		assert_noop!(Nicks::kill_name(Origin::signed(2), 1), BadOrigin);
// 		assert_noop!(Nicks::force_name(Origin::signed(2), 1, b"Whatever".to_vec()), BadOrigin);
// 	});
// }
