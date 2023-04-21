use crate::mock::*;
use frame_support::{assert_ok, assert_noop, assert_err, traits::Currency};
use gafi_support::common::constant::ID;
use gafi_support::pool::ticket::PlayerTicket;
use gafi_support::{
    common::currency::{unit, NativeToken::GAKI},
};
use sp_core::H160;
use sp_runtime::{AccountId32, Permill};
use sp_std::vec::Vec;
use crate::mock::Test;

const CIRCLE_BLOCK: u64 = (TIME_SERVICE as u64) / SLOT_DURATION;
const ADD_BLOCK: u64 = 1_u64;

fn new_account(account: [u8; 32], balance: u128) -> AccountId32 {
    let acc: AccountId32 = AccountId32::from(account.clone());
    let _ = pallet_balances::Pallet::<Test>::deposit_creating(&acc, balance);
    assert_eq!(Balances::free_balance(&acc), balance);
    return acc;
}

fn create_pool(
    account: AccountId32,
    targets: Vec<H160>,
    pool_value: u128,
    tx_limit: u32,
    discount: Permill,
) -> ID {
    let before_balance = Balances::free_balance(&account);
    assert_ok!(FundingPool::create_pool(
        RuntimeOrigin::signed(account.clone()),
        targets,
        pool_value,
        discount,
        tx_limit
    ));
    assert_eq!(
        Balances::free_balance(&account),
        before_balance - pool_value
    );

    let pool_id = FundingPool::pool_owned(&account);

    let id = pool_id.clone();

    *id.last().unwrap()
}

#[test]
fn create_funding_pool_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(1);
        let account_balance = one_mil_gaki();
        let account = new_account([0_u8; 32], account_balance);
        let targets = vec![H160::default()];
        let pool_value = 1000 * unit(GAKI);
        let tx_limit = 100_u32;
        let discount = Permill::from_percent(30);

        create_pool(account, targets, pool_value, tx_limit, discount);
    })
}

#[test]
fn rejoin_funding_pool_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(ADD_BLOCK);
        let account_balance = one_mil_gaki();
        let account = new_account([0_u8; 32], account_balance);
        let targets = vec![H160::default()];
        let pool_value = 1000 * unit(GAKI);
        let tx_limit = 100_u32;
        let discount = Permill::from_percent(30);

        let pool_id = create_pool(account, targets, pool_value, tx_limit, discount);

        let account_1 = new_account([1_u8; 32], account_balance);
        assert_ok!(Pool::join(
            RuntimeOrigin::signed(account_1.clone()),
            pool_id
        ));

        Pool::use_ticket(account_1.clone(), Some(H160::default()));
        Pool::use_ticket(account_1.clone(), Some(H160::default()));
        assert_eq!(Pool::tickets(account_1.clone(), pool_id).unwrap().tickets, 98_u32);

        run_to_block(10);
        assert_ok!(Pool::leave(RuntimeOrigin::signed(account_1.clone()), pool_id));
        assert_ok!(Pool::join(
            RuntimeOrigin::signed(account_1.clone()),
            pool_id
        ));
        assert_eq!(Pool::tickets(account_1.clone(), pool_id).unwrap().tickets, 98_u32);

        run_to_block(CIRCLE_BLOCK + ADD_BLOCK);
        assert_ok!(Pool::leave(RuntimeOrigin::signed(account_1.clone()), pool_id));
        assert_ok!(Pool::join(
            RuntimeOrigin::signed(account_1.clone()),
            pool_id
        ));
        assert_eq!(Pool::tickets(account_1.clone(), pool_id).unwrap().tickets, 100_u32);
    })
}

#[test]
fn limit_join_funding_pool_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(ADD_BLOCK);
        let account_balance = one_mil_gaki();
        let account = new_account([0_u8; 32], account_balance);
        let targets = vec![H160::default()];
        let pool_value = 1000 * unit(GAKI);
        let tx_limit = 100_u32;
        let discount = Permill::from_percent(30);

		let account_1 = new_account([1_u8; 32], account_balance);
		for i in 1..7 {
			run_to_block(i + 1);
			let pool_id = create_pool(account.clone(), targets.clone(), pool_value, tx_limit, discount);
			assert_ok!(Pool::join(
				RuntimeOrigin::signed(account_1.clone()),
				pool_id
			));
		}
		run_to_block(10);
		let pool_id1 = create_pool(account.clone(), targets, pool_value, tx_limit, discount);
		assert_noop!(Pool::join(
			RuntimeOrigin::signed(account_1.clone()),
			pool_id1
		), pallet_pool::Error::<Test>::ExceedJoinedPool);
    })
}
