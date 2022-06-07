use crate::mock::*;
use frame_support::{assert_ok, traits::Currency};
use gafi_primitives::{
    currency::{unit, NativeToken::GAKI},
    ticket::{PlayerTicket, SystemTicket, TicketLevel, TicketType},
};
use gafi_tx::Config;
use sp_runtime::AccountId32;
use sp_std::str::FromStr;
const TICKETS: [TicketType; 6] = [
    TicketType::System(SystemTicket::Upfront(TicketLevel::Basic)),
    TicketType::System(SystemTicket::Upfront(TicketLevel::Medium)),
    TicketType::System(SystemTicket::Upfront(TicketLevel::Advance)),
    TicketType::System(SystemTicket::Staking(TicketLevel::Basic)),
    TicketType::System(SystemTicket::Staking(TicketLevel::Medium)),
    TicketType::System(SystemTicket::Staking(TicketLevel::Advance)),
];

const LEVELS: [TicketLevel; 3] = [
    TicketLevel::Basic,
    TicketLevel::Medium,
    TicketLevel::Advance,
];

const CIRCLE_BLOCK: u64 = (TIME_SERVICE as u64) / SLOT_DURATION;
const ADDITIONAL_BLOCK: u64 = 1;

fn use_tickets(ticket: TicketType, account: AccountId32) {
    let base_balance = 1_000_000 * unit(GAKI);
    let _ = <Test as Config>::Currency::deposit_creating(&account, base_balance);
    assert_eq!(
        <Test as Config>::Currency::free_balance(account.clone()),
        base_balance
    );

    assert_ok!(Pool::join(Origin::signed(account.clone()), ticket));

    let service = Pool::get_service(ticket).unwrap();
    for _ in 0..service.tx_limit {
        assert_ne!(Pool::use_ticket(account.clone()), None);
    }
    assert_eq!(Pool::use_ticket(account.clone()), None);
}

#[test]
fn use_tickets_works() {
    ExtBuilder::default().build_and_execute(|| {
        for i in 0..TICKETS.len() {
            use_tickets(TICKETS[i], AccountId32::new([i as u8; 32]));
        }
    })
}

#[test]
fn renew_upfront_ticket_works() {
    for i in 0..LEVELS.len() {
        ExtBuilder::default().build_and_execute(|| {
            run_to_block(1);
            let account = AccountId32::new([i as u8; 32]);
            use_tickets(TicketType::System(SystemTicket::Upfront(LEVELS[i])), account.clone());
            assert_eq!(Pool::use_ticket(account.clone()), None);
            Pool::renew_tickets();
            assert_ne!(Pool::use_ticket(account.clone()), None);
        });
    }
}

#[test]
fn trigger_renew_upfront_tickets_works() {
    for i in 0..LEVELS.len() {
        ExtBuilder::default().build_and_execute(|| {
            run_to_block(1);
            let account = AccountId32::new([i as u8; 32]);
            use_tickets(TicketType::System(SystemTicket::Upfront(LEVELS[i])), account.clone());
            assert_eq!(Pool::use_ticket(account.clone()), None);
            run_to_block(CIRCLE_BLOCK + ADDITIONAL_BLOCK);
            assert_ne!(Pool::use_ticket(account.clone()), None);
        });
    }
}

#[test]
fn renew_staking_ticket_works() {
    for i in 0..LEVELS.len() {
        ExtBuilder::default().build_and_execute(|| {
            run_to_block(1);
            let account = AccountId32::new([i as u8; 32]);
            use_tickets(TicketType::System(SystemTicket::Staking(LEVELS[i])), account.clone());
            assert_eq!(Pool::use_ticket(account.clone()), None);
            Pool::renew_tickets();
            assert_ne!(Pool::use_ticket(account.clone()), None);
        });
    }
}

#[test]
fn trigger_renew_staking_tickets_works() {
    for i in 0..LEVELS.len() {
        ExtBuilder::default().build_and_execute(|| {
            run_to_block(1);
            let account = AccountId32::new([i as u8; 32]);
            use_tickets(TicketType::System(SystemTicket::Upfront(LEVELS[i])), account.clone());
            assert_eq!(Pool::use_ticket(account.clone()), None);
            Pool::renew_tickets();
            assert_ne!(Pool::use_ticket(account.clone()), None);
        });
    }
}
