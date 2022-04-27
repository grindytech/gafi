use crate::{mock::*, Error, PoolOwned, Pools};
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::constant::ID;
use gafi_primitives::currency::{unit, NativeToken::GAKI};
use gafi_primitives::pool::{StaticService, StaticPool};
use sp_runtime::AccountId32;
use sp_core::H160;
use sp_std::str::FromStr;

const CIRCLE_BLOCK: u64 = (TIME_SERVICE as u64) / SLOT_DURATION;

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
        create_pool(account, account_balance, vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()], pool_value, 10, 100);
    })
}

#[test]
fn create_pool_fail() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(1);

        let account_balance = 999 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);

        let pool_value = 1000 * unit(GAKI);
        assert_err!(
            Sponsored::create_pool(Origin::signed(account.clone()), vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()], pool_value, 10, 100),
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
        let pool_id = create_pool(account.clone(),  account_balance, vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()], pool_value, 10, 100);

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
            let pool_id = create_pool(account.clone(), account_balance, vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()], pool_value, 10, 100);
            assert_err!(
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
        let pool_id = create_pool(account.clone(), account_balance, vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()], pool_value, 100, 10);

        let service = Sponsored::get_service(pool_id).unwrap();
        assert_eq!(service.sponsor, account);
        assert_eq!(service.service.discount, 10);
        assert_eq!(service.service.tx_limit, 100);
        assert_eq!(service.targets, vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()]);
    })
}
