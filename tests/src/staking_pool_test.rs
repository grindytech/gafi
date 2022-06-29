use crate::mock::*;
use codec::Encode;
use frame_support::{assert_ok, traits::Currency};
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	ticket::{TicketLevel, TicketType, SystemTicket, CustomTicket},
};
use gafi_tx::Config;
use sp_io::hashing::blake2_256;
use sp_runtime::AccountId32;
use gafi_primitives::system_services::SystemPool;

const LEVELS: [TicketLevel; 3] = [TicketLevel::Basic, TicketLevel::Medium, TicketLevel::Advance];

fn join_pool(account: AccountId32, ticket: TicketType) {
	let base_balance = 1_000_000 * unit(GAKI);
	let pool_id =  match ticket {
		TicketType::System(system_ticket) => {
			system_ticket.using_encoded(blake2_256)
		}
		TicketType::Custom(CustomTicket::Sponsored(joined_pool_id)) => {
			joined_pool_id
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
		TicketType::System(system_ticket) => {
			system_ticket.using_encoded(blake2_256)
		}
		TicketType::Custom(CustomTicket::Sponsored(joined_pool_id)) => {
			joined_pool_id
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
    for i in 0..LEVELS.len() {
        ExtBuilder::default().build_and_execute(|| {
            let account = AccountId32::new([i as u8; 32]);

            join_pool(account, TicketType::System(SystemTicket::Staking(LEVELS[i])));
        })
    }
}


#[test]
fn leave_pool_works() {
    for i in 0..LEVELS.len() {
        ExtBuilder::default().build_and_execute(|| {
            let account = AccountId32::new([i as u8; 32]);
			let ticket = TicketType::System(SystemTicket::Staking(LEVELS[i]));

            join_pool(account.clone(), ticket);
            leave_pool(account.clone(),ticket);
        })
    }
}
