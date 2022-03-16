//! Benchmarking setup for pallet-pool

use super::*;
use crate::pool::PackService;
#[allow(unused)]
use crate::Pallet as Pool;
use crate::{Call, Config};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_benchmarking::Box;
use frame_system::RawOrigin;
use frame_support::traits::Currency;
use scale_info::prelude::string::String;
use scale_info::prelude::format;

const PACKS: [PackService; 3] = [PackService::Basic, PackService::Medium, PackService::Max];

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}


fn new_funded_account<T: Config>(index: u32, seed: u32, amount: u64) -> T::AccountId {
	let balance_amount = amount.try_into().ok().unwrap();
	let name: String = format!("{}{}", index, seed);
	// let user = account(string_to_static_str(name), index, seed);
	let user = account("caller", index, seed);
	T::Currency::make_free_balance_be(&user, balance_amount);
	T::Currency::issue(balance_amount);
	return user;
}

benchmarks! {
	join {
		let s in 0 .. 3;
		let caller: T::AccountId = new_funded_account::<T>(s, s, 1000_000u64);
	}: _(RawOrigin::Signed(new_funded_account::<T>(s, s, 1000_000u64)), PACKS[s as usize])
	verify {
		// assert_eq!(::<T>::get(), Some(s));
	}
}

