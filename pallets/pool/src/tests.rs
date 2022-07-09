use crate::{mock::*, Error, Tickets};
use frame_support::{assert_ok, traits::Currency};
use gafi_primitives::{
    constant::ID,
    currency::{unit, NativeToken::GAKI},
    ticket::{CustomTicket, SystemTicket, TicketLevel, TicketType},
};
use sp_core::H160;
use sp_runtime::{AccountId32, Permill};
use sponsored_pool::{PoolOwned, Pools};
use std::str::FromStr;

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
fn join_staking_pool_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(1);
        let account_balance = 1_000_000 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);

        Pool::join(
            Origin::signed(account.clone()),
            TicketType::System(SystemTicket::Staking(TicketLevel::Basic)),
        );

        assert_eq!(
            Balances::free_balance(account),
            account_balance - 1000 * unit(GAKI)
        );
    })
}

#[test]
fn leave_all_system_pool_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(1);
        let account_balance = 1_000_000 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);
        Pool::join(
            Origin::signed(account.clone()),
            TicketType::System(SystemTicket::Staking(TicketLevel::Basic)),
        );
        assert_ok!(Pool::leave_all(Origin::signed(account.clone())));

        assert_eq!(
            Tickets::<Test>::iter_prefix_values(account.clone()).count(),
            0
        );

        Pool::join(
            Origin::signed(account.clone()),
            TicketType::System(SystemTicket::Upfront(TicketLevel::Basic)),
        );
        assert_ok!(Pool::leave_all(Origin::signed(account.clone())));

        assert_eq!(
            Tickets::<Test>::iter_prefix_values(account.clone()).count(),
            0
        );
    })
}

fn create_pool(
    account: AccountId32,
    targets: Vec<H160>,
    pool_value: u128,
    tx_limit: u32,
    discount: Permill,
) -> ID {
    let account_balance: u128 = Balances::free_balance(&account);
    assert_ok!(SponsoredPool::create_pool(
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
    let new_pool = Pools::<Test>::get(pool_owned[pool_owned.len() - 1]).unwrap();
    assert_eq!(new_pool.owner, account);
    assert_eq!(new_pool.tx_limit, tx_limit);
    assert_eq!(new_pool.discount, discount);
    new_pool.id
}

#[test]
fn leave_all_custom_pool_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(1);
        let account_balance = 1_000_000 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);
        let pool_value = 1000 * unit(GAKI);

        let account2 = new_account([1_u8; 32], account_balance);
        {
            let pool_id = create_pool(
                account.clone(),
                vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
                pool_value,
                10,
                Permill::from_percent(70),
            );
            assert_ok!(Pool::join(
                Origin::signed(account2.clone()),
                TicketType::Custom(CustomTicket::Sponsored(pool_id))
            ));
        }

        // next random value
        run_to_block(2);
        {
            let pool_id = create_pool(
                account.clone(),
                vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
                pool_value,
                10,
                Permill::from_percent(70),
            );
            assert_ok!(Pool::join(
                Origin::signed(account2.clone()),
                TicketType::Custom(CustomTicket::Sponsored(pool_id))
            ));
        }

        assert_ok!(Pool::leave_all(Origin::signed(account2.clone())));
        assert_eq!(PoolOwned::<Test>::get(account2.clone()), [].to_vec());
        assert_eq!(Tickets::<Test>::iter_prefix_values(account2.clone()).count(), 0);
    })
}
