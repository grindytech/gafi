//! Benchmarking setup for pallet-pool

use super::*;
#[allow(unused)]
use crate::Pallet as Pool;
use crate::{Call, Config};
use frame_benchmarking::Box;
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use gafi_support::pool::{TicketType};
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

pub const STAKING_BASIC_ID: ID = [0_u8; 32];
pub const STAKING_MEDIUM_ID: ID = [1_u8; 32];
pub const STAKING_ADVANCE_ID: ID = [2_u8; 32];

pub const UPFRONT_BASIC_ID: ID = [10_u8; 32];
pub const UPFRONT_MEDIUM_ID: ID = [11_u8; 32];
pub const UPFRONT_ADVANCE_ID: ID = [12_u8; 32];

const TICKETS: [ID; MAX_TICKETS] = [
	STAKING_BASIC_ID,
	STAKING_MEDIUM_ID,
	STAKING_ADVANCE_ID,
	UPFRONT_BASIC_ID,
	UPFRONT_MEDIUM_ID,
	UPFRONT_ADVANCE_ID,
	POOL_ID,
];

benchmarks! {
	join {
		let s in 0 .. (MAX_TICKETS - 1) as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		T::FundingPool::add_default(caller.clone(), POOL_ID);
	}: _(RawOrigin::Signed(caller), TICKETS[s as usize])

	leave {
		let s in 0 .. (MAX_TICKETS - 1) as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		T::FundingPool::add_default(caller.clone(), POOL_ID);
		let pool_id = TICKETS[s as usize];
		let _ = Pallet::<T>::join(RawOrigin::Signed(caller.clone()).into(), pool_id);
	}: _(RawOrigin::Signed(caller), pool_id)


	leave_all {
		let s in 0 .. (MAX_TICKETS - 1) as u32;
		let caller = new_funded_account::<T>(s, s, 100_000_000u128 * UNIT);
		T::FundingPool::add_default(caller.clone(), POOL_ID);
		let pool_id = TICKETS[s as usize];
		let _ = Pallet::<T>::join(RawOrigin::Signed(caller.clone()).into(), pool_id);
	}: _(RawOrigin::Signed(caller))

	impl_benchmark_test_suite!(Pool, crate::mock::new_test_ext(), crate::mock::Test);
}
