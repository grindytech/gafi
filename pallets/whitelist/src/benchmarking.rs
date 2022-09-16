//! Benchmarking setup for pallet-template

use super::*;
#[allow(unused)]
use crate::Pallet as Whitelist;
use crate::{Call, Config};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

use frame_benchmarking::Box;
use frame_support::traits::Currency;
use scale_info::prelude::{format, string::String};

const UNIT: u128 = 1_000_000_000_000_000_000u128;
const POOL_ID: ID = [0_u8; 32];
const MAX: u32 = 100;

fn string_to_static_str(s: String) -> &'static str {
	Box::leak(s.into_boxed_str())
}

fn new_funded_account<T: Config>(index: u32, seed: u32, amount: u128) -> T::AccountId {
	let balance_amount = amount.try_into().ok().unwrap();
	let name: String = format!("{}{}", index, seed);
	let user = account(string_to_static_str(name), index, seed);
	T::Currency::make_free_balance_be(&user, balance_amount);
	T::Currency::issue(balance_amount);
	return user
}

benchmarks! {
	enable_whitelist {
		let s in 0 .. MAX;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		T::SponsoredPool::add_default(caller.clone(), POOL_ID);
		let url = b"http://whitelist.gafi.network/whitelist/verify";
	}: _(RawOrigin::Signed(caller), POOL_ID, url.to_vec())


	apply_whitelist {
		let s in 0 .. MAX;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		let player = new_funded_account::<T>(s + MAX, s + MAX, 1000_000_000u128 * UNIT);
		T::SponsoredPool::add_default(caller.clone(), POOL_ID);
		let url = b"http://whitelist.gafi.network/whitelist/verify";

		Whitelist::<T>::enable_whitelist(RawOrigin::Signed(caller.clone()).into(), POOL_ID, url.to_vec());

	}: _(RawOrigin::Signed(player), POOL_ID)

	approve_whitelist {
		let s in 0 .. MAX;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		let player = new_funded_account::<T>(s + MAX , s + MAX, 1000_000_000u128 * UNIT);
		T::SponsoredPool::add_default(caller.clone(), POOL_ID);

		let url = b"http://whitelist.gafi.network/whitelist/verify";

		Whitelist::<T>::enable_whitelist(RawOrigin::Signed(caller.clone()).into(), POOL_ID, url.to_vec());

		let _ = Whitelist::<T>::apply_whitelist(RawOrigin::Signed(player.clone()).into(), POOL_ID);

	}: _(RawOrigin::Signed(caller), player,  POOL_ID)

	approve_whitelist_unsigned {
		let s in 0 .. MAX;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		let player = new_funded_account::<T>(s + MAX , s + MAX, 1000_000_000u128 * UNIT);
		T::SponsoredPool::add_default(caller.clone(), POOL_ID);

		let url = b"http://whitelist.gafi.network/whitelist/verify";

		Whitelist::<T>::enable_whitelist(RawOrigin::Signed(caller.clone()).into(), POOL_ID, url.to_vec());

		let _ = Whitelist::<T>::apply_whitelist(RawOrigin::Signed(player.clone()).into(), POOL_ID);

	}: _(RawOrigin::None, player,  POOL_ID)

	withdraw_whitelist {
		let s in 0 .. MAX;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		T::SponsoredPool::add_default(caller.clone(), POOL_ID);
		let url = b"http://whitelist.gafi.network/whitelist/verify";

		Whitelist::<T>::enable_whitelist(RawOrigin::Signed(caller.clone()).into(), POOL_ID, url.to_vec());

	}: _(RawOrigin::Signed(caller), POOL_ID)

	impl_benchmark_test_suite!(Whitelist, crate::mock::new_test_ext(), crate::mock::Test);
}
