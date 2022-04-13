//! Benchmarking setup for pallet-pool

use super::*;
#[allow(unused)]
use crate::Pallet as Pool;
use crate::{Call, Config};
use frame_benchmarking::{account, benchmarks};
use frame_benchmarking::Box;
use frame_system::RawOrigin;
use frame_support::traits::Currency;
use scale_info::prelude::string::String;
use scale_info::prelude::format;
use gafi_primitives::{pool::{Level, TicketType}};
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
const MAX_TICKETS: usize = 6;

const TICKETS: [TicketType; MAX_TICKETS] = [TicketType::Upfront(Level::Basic),
 TicketType::Upfront(Level::Medium),
TicketType::Upfront(Level::Advance),
TicketType::Staking(Level::Basic),
 TicketType::Staking(Level::Medium),
TicketType::Staking(Level::Advance),
];

benchmarks! {
	join {
		let s in 0 .. (MAX_TICKETS - 1) as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
	}: _(RawOrigin::Signed(caller), TICKETS[s as usize])

	leave {
		let s in 0 .. (MAX_TICKETS - 1) as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		Pallet::<T>::join(RawOrigin::Signed(caller.clone()).into(), TICKETS[s as usize]);
	}: _(RawOrigin::Signed(caller))

}