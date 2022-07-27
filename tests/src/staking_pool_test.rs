use crate::mock::*;
use codec::Encode;
use frame_support::{assert_ok, traits::Currency};
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI}, ticket::TicketType,
};
use gafi_tx::Config;
use sp_io::hashing::blake2_256;
use sp_runtime::AccountId32;
use gafi_primitives::system_services::SystemPool;
const TICKETS: [TicketType; 3] = [
    TicketType::Staking(STAKING_BASIC_ID),
    TicketType::Staking(STAKING_MEDIUM_ID),
    TicketType::Staking(STAKING_ADVANCE_ID),
];

fn join_pool(account: AccountId32, ticket: TicketType) {
	let base_balance = 1_000_000 * unit(GAKI);
	let pool_id =  match ticket {
		TicketType::Sponsored(id) |
        TicketType::Staking(id) |
         TicketType::Upfront(id) => {
			id
		}
	};
	let staking_amount = StakingPool::get_service(pool_id).unwrap().value;
	let _ = <Test as Config>::Currency::deposit_creating(&account, base_balance);

	{
		assert_eq!(<Test as Config>::Currency::free_balance(account.clone()), base_balance);
	}

	assert_ok!(Pool::join(Origin::signed(account.clone()), ticket));
	assert_eq!(
		<Test as Config>::Currency::free_balance(account.clone()),
		base_balance - staking_amount
	);
}

fn leave_pool(account: AccountId32, ticket: TicketType) {
    let before_balance = <Test as Config>::Currency::free_balance(account.clone());
	let pool_id =  match ticket {
		TicketType::Sponsored(id) |
        TicketType::Staking(id) |
         TicketType::Upfront(id) => {
			id
		}
	};
	let staking_amount = StakingPool::get_service(pool_id).unwrap().value;

	assert_ok!(Pool::leave(Origin::signed(account.clone()), pool_id));
	assert_eq!(
		<Test as Config>::Currency::free_balance(account.clone()),
		before_balance + staking_amount
	);
}

#[test]
fn join_pool_works() {
    for i in 0..TICKETS.len() {
        ExtBuilder::default().build_and_execute(|| {
            let account = AccountId32::new([i as u8; 32]);

            join_pool(account, TICKETS[i]);
        })
    }
}


#[test]
fn leave_pool_works() {
    for i in 0..TICKETS.len() {
        ExtBuilder::default().build_and_execute(|| {
            let account = AccountId32::new([i as u8; 32]);
			let ticket = TICKETS[i];

            join_pool(account.clone(), ticket);
            leave_pool(account.clone(),ticket);
        })
    }
}
