use crate::mock::*;
use frame_support::{assert_ok, traits::Currency};
use gafi_primitives::{
    currency::{unit, NativeToken::GAKI},
    ticket::{PlayerTicket, TicketType},
};
use gafi_tx::Config;
use sp_runtime::AccountId32;
use gafi_primitives::constant::ID;

const TICKETS: [ID; 6] = [
    UPFRONT_BASIC_ID,
    UPFRONT_MEDIUM_ID,
    UPFRONT_ADVANCE_ID,
    STAKING_BASIC_ID,
    STAKING_MEDIUM_ID,
    STAKING_ADVANCE_ID,
];

const CIRCLE_BLOCK: u64 = (TIME_SERVICE as u64) / SLOT_DURATION;
const ADDITIONAL_BLOCK: u64 = 1;

fn use_tickets(pool_id: ID, account: AccountId32) {
    let base_balance = 1_000_000 * unit(GAKI);
	
    let _ = <Test as Config>::Currency::deposit_creating(&account, base_balance);

    assert_eq!(
        <Test as Config>::Currency::free_balance(account.clone()),
        base_balance
    );
    assert_ok!(Pool::join(Origin::signed(account.clone()), pool_id));

    let service = Pool::get_service(pool_id).unwrap();

    for _ in 0..service.tx_limit {
        assert_ne!(Pool::use_ticket(account.clone(), None), None);
    }
    assert_eq!(Pool::use_ticket(account.clone(), None), None);
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
    for i in 0..TICKETS.len() {
        ExtBuilder::default().build_and_execute(|| {
            run_to_block(1);
            let account = AccountId32::new([i as u8; 32]);
            use_tickets(TICKETS[i], account.clone());
            assert_eq!(Pool::use_ticket(account.clone(), None), None);
            Pool::renew_tickets();
            assert_ne!(Pool::use_ticket(account.clone(), None), None);
        });
    }
}

#[test]
fn trigger_renew_upfront_tickets_works() {
    for i in 0..TICKETS.len() {
        ExtBuilder::default().build_and_execute(|| {
            run_to_block(1);
            let account = AccountId32::new([i as u8; 32]);
            use_tickets(TICKETS[i], account.clone());
            assert_eq!(Pool::use_ticket(account.clone(), None), None);
            run_to_block(CIRCLE_BLOCK + ADDITIONAL_BLOCK);
            assert_ne!(Pool::use_ticket(account.clone(), None), None);
        });
    }
}

#[test]
fn renew_staking_ticket_works() {
    for i in 0..TICKETS.len() {
        ExtBuilder::default().build_and_execute(|| {
            run_to_block(1);
            let account = AccountId32::new([i as u8; 32]);
            use_tickets(TICKETS[i], account.clone());
            assert_eq!(Pool::use_ticket(account.clone(), None), None);
            Pool::renew_tickets();
            assert_ne!(Pool::use_ticket(account.clone(), None), None);
        });
    }
}

#[test]
fn trigger_renew_staking_tickets_works() {
    for i in 0..TICKETS.len() {
        ExtBuilder::default().build_and_execute(|| {
            run_to_block(1);
            let account = AccountId32::new([i as u8; 32]);
            use_tickets(TICKETS[i], account.clone());
            assert_eq!(Pool::use_ticket(account.clone(), None), None);
            Pool::renew_tickets();
            assert_ne!(Pool::use_ticket(account.clone(), None), None);
        });
    }
}
