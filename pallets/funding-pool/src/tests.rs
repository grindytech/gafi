use crate::{mock::*, Error, PoolOwned, Pools, Targets};
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use gafi_primitives::{
	common::{constant::ID,
	currency::{unit, NativeToken::GAKI}},
	pool::custom_services::CustomPool,
};
use sp_core::H160;
use sp_runtime::{AccountId32, Permill};
use sp_std::{str::FromStr, vec::Vec};

fn make_deposit(account: &AccountId32, balance: u128) {
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(account: [u8; 32], balance: u128) -> AccountId32 {
	let acc: AccountId32 = AccountId32::from(account);
	make_deposit(&acc, balance);
	assert_eq!(Balances::free_balance(&acc), balance);
	return acc
}

fn create_pool(
	account: AccountId32,
	account_balance: u128,
	targets: Vec<H160>,
	pool_value: u128,
	tx_limit: u32,
	discount: Permill,
) -> ID {
	assert_ok!(Funding::create_pool(
		RuntimeOrigin::signed(account.clone()),
		targets,
		pool_value,
		discount,
		tx_limit
	));

	assert_eq!(
		Balances::free_balance(&account),
		account_balance - pool_value
	);
	let pool_owned = PoolOwned::<Test>::get(account.clone());
	assert_eq!(pool_owned.len(), 1);
	let new_pool = Pools::<Test>::get(pool_owned[0]).unwrap();
	assert_eq!(new_pool.owner, account);
	assert_eq!(new_pool.tx_limit, tx_limit);
	assert_eq!(new_pool.discount, discount);
	new_pool.id
}

#[test]
fn new_pool_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);

		let new_pool = Funding::new_pool();
		assert_eq!(new_pool.unwrap().id.len(), 32);
	})
}

#[test]
fn create_pool_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
		let account = new_account([0_u8; 32], account_balance);
		let pool_value = 1000 * unit(GAKI);
		create_pool(
			account.clone(),
			account_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
			pool_value,
			10,
			Permill::from_percent(70),
		);

		let pool_id: ID = *PoolOwned::<Test>::get(account.clone()).last().unwrap();
		let pool_acc = AccountId32::from(pool_id);

		assert_eq!(Balances::free_balance(pool_acc), pool_value);
	})
}

#[test]
fn create_pool_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);

		let account_balance = 999 * unit(GAKI);
		let account = new_account([0_u8; 32], account_balance);

		let pool_value = 1000 * unit(GAKI);
		assert_noop!(
			Funding::create_pool(
				RuntimeOrigin::signed(account.clone()),
				vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
				pool_value,
				Permill::from_percent(10),
				100
			),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	})
}

#[test]
fn withdraw_pool_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
		let account = new_account([0_u8; 32], account_balance);
		let pool_value = 1000 * unit(GAKI);
		let pool_id = create_pool(
			account.clone(),
			account_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
			pool_value,
			10,
			Permill::from_percent(70),
		);

		assert_ok!(Funding::withdraw_pool(
			RuntimeOrigin::signed(account.clone()),
			pool_id
		));
		assert_eq!(Balances::free_balance(&account), account_balance);
	})
}

#[test]
fn withdraw_pool_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
		let account = new_account([0_u8; 32], account_balance);
		let pool_value = 1000 * unit(GAKI);
		// pool_id not exist
		{
			let new_pool = Funding::new_pool();
			assert_err!(
				Funding::withdraw_pool(
					RuntimeOrigin::signed(account.clone()),
					new_pool.unwrap().id
				),
				Error::<Test>::PoolNotExist
			);
		}

		// not the owner
		{
			let account_1 = new_account([1_u8; 32], account_balance);
			let pool_id = create_pool(
				account.clone(),
				account_balance,
				vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
				pool_value,
				10,
				Permill::from_percent(10),
			);
			assert_noop!(
				Funding::withdraw_pool(RuntimeOrigin::signed(account_1.clone()), pool_id),
				Error::<Test>::NotTheOwner
			);
		}
	})
}

#[test]
fn get_service_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
		let account = new_account([0_u8; 32], account_balance);
		let pool_value = 1000 * unit(GAKI);
		let pool_id = create_pool(
			account.clone(),
			account_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
			pool_value,
			100,
			Permill::from_percent(10),
		);

		let service = Funding::get_service(pool_id).unwrap();
		assert_eq!(service.sponsor, account);
		assert_eq!(service.service.discount, Permill::from_percent(10));
		assert_eq!(service.service.tx_limit, 100);
		assert_eq!(
			service.targets,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()]
		);
	})
}

#[test]
fn new_targets_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
		let account = new_account([0_u8; 32], account_balance);
		let pool_value = 1000 * unit(GAKI);
		create_pool(
			account.clone(),
			account_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
			pool_value,
			10,
			Permill::from_percent(70),
		);

		let pool_id: ID = *PoolOwned::<Test>::get(account.clone()).last().unwrap();
		let new_targets: Vec<H160> =
			vec![H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap()];

		assert_ok!(Funding::new_targets(
			RuntimeOrigin::signed(account.clone()),
			pool_id,
			new_targets.clone()
		));
		let targets = Targets::<Test>::get(pool_id);

		assert_eq!(targets, new_targets);
	})
}

#[test]
fn new_targets_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
		let account = new_account([0_u8; 32], account_balance);
		let pool_value = 1000 * unit(GAKI);
		let pool_id;
		// pool_id not exist
		{
			let new_pool = Funding::new_pool();
			assert_noop!(
				Funding::withdraw_pool(
					RuntimeOrigin::signed(account.clone()),
					new_pool.unwrap().id
				),
				Error::<Test>::PoolNotExist
			);
		}

		// not the owner
		{
			let account_1 = new_account([1_u8; 32], account_balance);
			pool_id = create_pool(
				account.clone(),
				account_balance,
				vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
				pool_value,
				10,
				Permill::from_percent(70),
			);
			assert_noop!(
				Funding::withdraw_pool(RuntimeOrigin::signed(account_1.clone()), pool_id),
				Error::<Test>::NotTheOwner
			);
		}

		// exceed pool target
		{
			let new_targets: Vec<H160> = vec![
				H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
				H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
				H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
				H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
				H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
				H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
				H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
				H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
				H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
				H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
				H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
			];
			assert_noop!(
				Funding::new_targets(
					RuntimeOrigin::signed(account.clone()),
					pool_id,
					new_targets.clone()
				),
				<Error<Test>>::ExceedPoolTarget
			);
		}
	})
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
			account_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
			pool_value,
			10,
			Permill::from_percent(70),
		);
		let free_balance = pool_value - RESERVATION_FEE * unit(GAKI);

		assert_ok!(Funding::set_pool_name(
			RuntimeOrigin::signed(account.clone()),
			pool_id,
			b"Test pool".to_vec()
		));
		assert_eq!(
			Balances::reserved_balance(Funding::to_account(pool_id).unwrap()),
			RESERVATION_FEE * unit(GAKI)
		);
		assert_eq!(
			Balances::free_balance(Funding::to_account(pool_id).unwrap()),
			free_balance
		);

		assert_ok!(Funding::set_pool_name(
			RuntimeOrigin::signed(account.clone()),
			pool_id,
			b"Test pool1".to_vec()
		));
		assert_eq!(
			Balances::reserved_balance(Funding::to_account(pool_id).unwrap()),
			RESERVATION_FEE * unit(GAKI)
		);
		assert_eq!(
			Balances::free_balance(Funding::to_account(pool_id).unwrap()),
			free_balance
		);

		assert_ok!(Funding::clear_pool_name(
			RuntimeOrigin::signed(account.clone()),
			pool_id
		));
		assert_eq!(Balances::reserved_balance(account.clone()), 0);
		assert_eq!(
			Balances::free_balance(account.clone()),
			account_balance - pool_value
		);
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
			account_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
			pool_value,
			10,
			Permill::from_percent(70),
		);

		assert_ok!(Funding::set_pool_name(
			RuntimeOrigin::signed(account.clone()),
			pool_id,
			b"Test pool".to_vec()
		));
		assert_eq!(
			Balances::total_balance(&account),
			account_balance - pool_value
		);
		assert_eq!(
			Balances::reserved_balance(Funding::to_account(pool_id).unwrap()),
			RESERVATION_FEE * unit(GAKI)
		);
		assert_ok!(Funding::kill_pool_name(RuntimeOrigin::root(), pool_id));
		assert_eq!(
			Balances::total_balance(&Funding::to_account(pool_id).unwrap()),
			pool_value - RESERVATION_FEE * unit(GAKI)
		);
	});
}

#[test]
fn error_catching_should_work() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let account_balance = 1_001_000 * unit(GAKI);
		let account = new_account([0_u8; 32], account_balance);
		let account1_balance = 1001 * unit(GAKI);
		let account1 = new_account([1_u8; 32], account1_balance);
		let pool_value = 1000 * unit(GAKI);
		let pool_id = create_pool(
			account.clone(),
			account_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
			pool_value,
			10,
			Permill::from_percent(70),
		);
		run_to_block(2);
		let pool_id1 = create_pool(
			account1.clone(),
			account1_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84c").unwrap()],
			pool_value,
			20,
			Permill::from_percent(70),
		);

		assert_noop!(
			Funding::clear_pool_name(RuntimeOrigin::signed(account.clone()), pool_id),
			pallet_nicks::Error::<Test>::Unnamed
		);

		assert_noop!(
			Funding::set_pool_name(
				RuntimeOrigin::signed(account.clone()),
				pool_id,
				b"Te".to_vec()
			),
			pallet_nicks::Error::<Test>::TooShort
		);
		assert_noop!(
			Funding::set_pool_name(
				RuntimeOrigin::signed(account.clone()),
				pool_id,
				b"Test pool name with 16 chars".to_vec()
			),
			pallet_nicks::Error::<Test>::TooLong
		);
	});
}
