use crate::{mock::*};
use frame_support::{assert_ok, traits::Currency};
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	pool::{Level, TicketType, PlayerTicket},
};
use gafi_tx::Config;
use sp_runtime::AccountId32;
use sp_std::str::FromStr;
const TICKETS: [TicketType; 6] = [TicketType::Upfront(Level::Basic),
 TicketType::Upfront(Level::Medium), TicketType::Upfront(Level::Advance),
TicketType::Staking(Level::Basic), TicketType::Staking(Level::Medium),
 TicketType::Staking(Level::Advance)];
 
const CIRCLE_BLOCK: u64 = (TIME_SERVICE as u64) / SLOT_DURATION;
const ADDITIONAL_BLOCK: u64 = 1;

fn use_tickets(ticket: TicketType, account: AccountId32) {
    let base_balance = 1_000_000 * unit(GAKI);
    let _ = <Test as Config>::Currency::deposit_creating(&account, base_balance);
    assert_eq!(<Test as Config>::Currency::free_balance(account.clone()), base_balance);

    assert_ok!(Pool::join(Origin::signed(account.clone()), ticket));

    let service = Pool::get_service(ticket);
    for _ in 0..service.tx_limit {
        assert_ne!(Pool::use_ticket(account.clone()), None);
    }
    assert_eq!(Pool::use_ticket(account.clone()), None);
}


#[test]
fn use_tickets_works() {
    for i in 0..TICKETS.len() {
        ExtBuilder::default().build_and_execute(|| {
            use_tickets(TICKETS[i], AccountId32::new([i as u8;32]));
        })
    }
}

// #[test]
// fn renew_ticket_works() {
//     ExtBuilder::default().build_and_execute(|| {
//     for i in 0..TICKETS.len() {
//             run_to_block(1);
//             let account = AccountId32::new([i as u8;32]);
//             use_tickets(TicketType::Upfront(Level::Basic), account.clone());
            
//             run_to_block(CIRCLE_BLOCK + ADDITIONAL_BLOCK);
//             assert_ne!(Pool::use_ticket(account.clone()), None);
//         }
//     });
// }
