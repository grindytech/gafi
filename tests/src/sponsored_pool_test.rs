use crate::mock::*;
use frame_support::{assert_ok, traits::Currency};
use gafi_primitives::constant::ID;
use gafi_primitives::system_services::SystemPool;
use gafi_primitives::ticket::PlayerTicket;
use gafi_primitives::{
    currency::{unit, NativeToken::GAKI},
    ticket::{CustomTicket, TicketType},
};
use sp_core::H160;
use sp_runtime::{AccountId32, Permill};
use sp_std::vec::Vec;

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
    assert_ok!(SponsoredPool::create_pool(
        Origin::signed(account.clone()),
        targets,
        pool_value,
        discount,
        tx_limit
    ));
    assert_eq!(
        Balances::free_balance(&account),
        before_balance - pool_value
    );

    let pool_id = SponsoredPool::pool_owned(&account);

    let id = pool_id.clone();

    *id.last().unwrap()
}

#[test]
fn create_sponsored_pool_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(1);
        let account_balance = 1_000_000 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);
        let targets = vec![H160::default()];
        let pool_value = 1000 * unit(GAKI);
        let tx_limit = 100_u32;
        let discount = Permill::from_percent(30);

        create_pool(account, targets, pool_value, tx_limit, discount);
    })
}

#[test]
fn rejoin_sponsored_pool_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(ADD_BLOCK);
        let account_balance = 1_000_000 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);
        let targets = vec![H160::default()];
        let pool_value = 1000 * unit(GAKI);
        let tx_limit = 100_u32;
        let discount = Permill::from_percent(30);

        let pool_id = create_pool(account, targets, pool_value, tx_limit, discount);

        let account_1 = new_account([1_u8; 32], account_balance);
        assert_ok!(Pool::join(
            Origin::signed(account_1.clone()),
            TicketType::Custom(CustomTicket::Sponsored(pool_id))
        ));

        Pool::use_ticket(account_1.clone());
        Pool::use_ticket(account_1.clone());
        assert_eq!(Pool::tickets(account_1.clone()).unwrap().tickets, 98_u32);

        run_to_block(10);
        assert_ok!(Pool::leave(Origin::signed(account_1.clone().clone())));
        assert_ok!(Pool::join(
            Origin::signed(account_1.clone()),
            TicketType::Custom(CustomTicket::Sponsored(pool_id))
        ));
        assert_eq!(Pool::tickets(account_1.clone()).unwrap().tickets, 98_u32);

        run_to_block(CIRCLE_BLOCK + ADD_BLOCK);
        assert_ok!(Pool::leave(Origin::signed(account_1.clone().clone())));
        assert_ok!(Pool::join(
            Origin::signed(account_1.clone()),
            TicketType::Custom(CustomTicket::Sponsored(pool_id))
        ));
        assert_eq!(Pool::tickets(account_1.clone()).unwrap().tickets, 100_u32);
    })
}
