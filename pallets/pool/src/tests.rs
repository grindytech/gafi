use crate::{mock::*, Error, Tickets};
use frame_support::traits::Currency;
use gafi_primitives::{
    currency::{unit, NativeToken::GAKI},
    ticket::{SystemTicket, TicketLevel, TicketType},
};
use sp_runtime::AccountId32;

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
        Pool::leave_all(Origin::signed(account.clone()));

        assert_eq!(
            Tickets::<Test>::iter_prefix_values(account.clone()).count(),
            0
        );

        Pool::join(
            Origin::signed(account.clone()),
            TicketType::System(SystemTicket::Upfront(TicketLevel::Basic)),
        );
        Pool::leave_all(Origin::signed(account.clone()));

        assert_eq!(
            Tickets::<Test>::iter_prefix_values(account.clone()).count(),
            0
        );
    })
}



