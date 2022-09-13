//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use frame_benchmarking::account;

const UNIT: u128 = 1_000_000_000_000_000_000u128;
const POOL_ID: ID = [0_u8; 32];

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

benchmarks! {
	set_whitelist_url {
		let s in 0 .. 100;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		T::SponsoredPool::add_default(caller.clone(), POOL_ID);

	}: _(RawOrigin::Signed(caller), POOL_ID)
	
	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
