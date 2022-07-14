//! Benchmarking setup for pallet-pool

use super::*;
#[allow(unused)]
use crate::Pallet as Pool;
use crate::{Call, Config};
use frame_benchmarking::Box;
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use gafi_primitives::ticket::{TicketLevel, TicketType};
use scale_info::prelude::format;
use scale_info::prelude::string::String;
const UNIT: u128 = 1_000_000_000_000_000_000u128;

fn string_to_static_str(s: String) -> &'static str {
	Box::leak(s.into_boxed_str())
}

fn new_funded_account<T: Config>(index: u32, seed: u32, amount: u128) -> T::AccountId {
	let balance_amount = amount.try_into().ok().unwrap();
	let name: String = format!("{}{}", index, seed);
	let user = account(string_to_static_str(name), index, seed);
	T::Currency::make_free_balance_be(&user, balance_amount);
	T::Currency::issue(balance_amount);
	return user;
}

const MAX_TICKETS: usize = 7;
const POOL_ID: ID = [0_u8; 32];

const TICKETS: [TicketType; MAX_TICKETS] = [
	TicketType::System(SystemTicket::Upfront(TicketLevel::Basic)),
	TicketType::System(SystemTicket::Upfront(TicketLevel::Medium)),
	TicketType::System(SystemTicket::Upfront(TicketLevel::Advance)),
	TicketType::System(SystemTicket::Staking(TicketLevel::Basic)),
	TicketType::System(SystemTicket::Staking(TicketLevel::Medium)),
	TicketType::System(SystemTicket::Staking(TicketLevel::Advance)),
	TicketType::Custom(CustomTicket::Sponsored(POOL_ID)),
];

benchmarks! {
	join {
		let s in 0 .. (MAX_TICKETS - 1) as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		T::SponsoredPool::add_default(caller.clone(), POOL_ID);
	}: _(RawOrigin::Signed(caller), TICKETS[s as usize])

	leave {
		let s in 0 .. (MAX_TICKETS - 1) as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		let pool_id =  match TICKETS[s as usize] {
			TicketType::System(system_ticket) => {
				system_ticket.using_encoded(blake2_256)
			}
			TicketType::Custom(CustomTicket::Sponsored(joined_pool_id)) => {
				joined_pool_id
			}
		};
		T::SponsoredPool::add_default(caller.clone(), POOL_ID);
		let _ = Pallet::<T>::join(RawOrigin::Signed(caller.clone()).into(), TICKETS[s as usize]);
	}: _(RawOrigin::Signed(caller), pool_id)


	leave_all {
		let s in 0 .. (MAX_TICKETS - 1) as u32;
		let caller = new_funded_account::<T>(s, s, 100_000_000u128 * UNIT);
		let pool_id =  match TICKETS[s as usize] {
			TicketType::System(system_ticket) => {
				system_ticket.using_encoded(blake2_256)
			}
			TicketType::Custom(CustomTicket::Sponsored(joined_pool_id)) => {
				joined_pool_id
			}
		};
		T::SponsoredPool::add_default(caller.clone(), POOL_ID);
		let _ = Pallet::<T>::join(RawOrigin::Signed(caller.clone()).into(), TICKETS[s as usize]);
	}: _(RawOrigin::Signed(caller))

	impl_benchmark_test_suite!(Pool, crate::mock::new_test_ext(), crate::mock::Test);
}
