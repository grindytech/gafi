use crate::{mock::*, Error, PoolOwned, Pools, Targets};
use frame_support::assert_err;
use frame_support::{assert_noop, assert_ok, traits::Currency};
use gafi_primitives::constant::ID;
use gafi_primitives::currency::{unit, NativeToken::GAKI};
use gafi_primitives::custom_services::CustomePool;
use sp_core::H160;
use sp_runtime::AccountId32;
use sp_std::str::FromStr;
use sp_std::vec::Vec;

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
fn new_pool_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(1);

        let new_pool = Sponsored::new_pool();
        assert_eq!(new_pool.unwrap().id.len(), 32);
    })
}

fn create_pool(
    account: AccountId32,
    account_balance: u128,
    targets: Vec<H160>,
    pool_value: u128,
    tx_limit: u32,
    discount: u8,
) -> ID {
    assert_ok!(Sponsored::create_pool(
        Origin::signed(account.clone()),
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
            100,
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
            Sponsored::create_pool(
                Origin::signed(account.clone()),
                vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
                pool_value,
                10,
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
            100,
        );

        assert_ok!(Sponsored::withdraw_pool(
            Origin::signed(account.clone()),
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
            let new_pool = Sponsored::new_pool();
            assert_err!(
                Sponsored::withdraw_pool(Origin::signed(account.clone()), new_pool.unwrap().id),
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
                100,
            );
            assert_noop!(
                Sponsored::withdraw_pool(Origin::signed(account_1.clone()), pool_id),
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
            10,
        );

        let service = Sponsored::get_service(pool_id).unwrap();
        assert_eq!(service.sponsor, account);
        assert_eq!(service.service.discount, 10);
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
            100,
        );

        let pool_id: ID = *PoolOwned::<Test>::get(account.clone()).last().unwrap();
        let new_targets: Vec<H160> =
            vec![H160::from_str("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap()];

        assert_ok!(Sponsored::new_targets(
            Origin::signed(account.clone()),
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
            let new_pool = Sponsored::new_pool();
            assert_noop!(
                Sponsored::withdraw_pool(Origin::signed(account.clone()), new_pool.unwrap().id),
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
                100,
            );
            assert_noop!(
                Sponsored::withdraw_pool(Origin::signed(account_1.clone()), pool_id),
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
                Sponsored::new_targets(
                    Origin::signed(account.clone()),
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
            100,
        );
		let free_balance = account_balance - pool_value - RESERVATION_FEE * unit(GAKI);

		assert_ok!(Sponsored::set_pool_name(Origin::signed(account.clone()), pool_id, b"Test pool".to_vec()));
		assert_eq!(Balances::reserved_balance(account.clone()), RESERVATION_FEE * unit(GAKI));
		assert_eq!(Balances::free_balance(account.clone()), free_balance);
		assert_eq!(PoolNames::name_of(pool_id).unwrap().0, b"Test pool".to_vec());

		assert_ok!(Sponsored::set_pool_name(Origin::signed(account.clone()), pool_id, b"Test pool1".to_vec()));
		assert_eq!(Balances::reserved_balance(account.clone()), RESERVATION_FEE * unit(GAKI));
		assert_eq!(Balances::free_balance(account.clone()), free_balance);
		assert_eq!(pallet_pool_names::Pallet::<Test>::name_of(pool_id).unwrap().0, b"Test pool1".to_vec());

		assert_ok!(Sponsored::clear_pool_name(Origin::signed(account.clone()), pool_id));
		assert_eq!(Balances::reserved_balance(account.clone()), 0);
		assert_eq!(Balances::free_balance(account.clone()), account_balance - pool_value);
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
			100,
		);

		assert_ok!(Sponsored::set_pool_name(Origin::signed(account.clone()), pool_id, b"Test pool".to_vec()));
		assert_eq!(Balances::total_balance(&account), account_balance - pool_value);
		assert_eq!(Balances::reserved_balance(account.clone()), RESERVATION_FEE * unit(GAKI));
		assert_ok!(Sponsored::kill_pool_name(Origin::root(), pool_id));
		assert_eq!(Balances::total_balance(&account), account_balance - pool_value - RESERVATION_FEE * unit(GAKI));
		assert_eq!(pallet_pool_names::Pallet::<Test>::name_of(pool_id), None);
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
            100,
        );
		run_to_block(2);
		let pool_id1 = create_pool(
            account1.clone(),
			account1_balance,
            vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84c").unwrap()],
            pool_value,
            20,
            100,
        );

		assert_noop!(Sponsored::clear_pool_name(Origin::signed(account.clone()), pool_id), pallet_pool_names::Error::<Test>::Unnamed);

		assert_noop!(
			Sponsored::set_pool_name(Origin::signed(account1.clone()), pool_id1, b"Test pool".to_vec()),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);

		assert_noop!(
			Sponsored::set_pool_name(Origin::signed(account.clone()), pool_id, b"Te".to_vec()),
			pallet_pool_names::Error::<Test>::TooShort
		);
		assert_noop!(
			Sponsored::set_pool_name(Origin::signed(account.clone()), pool_id, b"Test pool name with 16 chars".to_vec()),
			pallet_pool_names::Error::<Test>::TooLong
		);


	});
}
