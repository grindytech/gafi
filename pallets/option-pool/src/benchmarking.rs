//! Benchmarking setup for pallet-option-pool

use super::*;
use crate::pool::PackService;
#[allow(unused)]
use crate::Pallet as Pool;
use crate::{Call, Config};
use frame_benchmarking::{account, benchmarks};
use frame_benchmarking::Box;
use frame_system::RawOrigin;
use frame_support::traits::Currency;
use scale_info::prelude::string::String;
use scale_info::prelude::format;


fn string_to_static_str(s: String) -> &'static str {
	Box::leak(s.into_boxed_str())
}


fn new_funded_account<T: Config>(index: u32, seed: u32, amount: u64) -> T::AccountId {
	let balance_amount = amount.try_into().ok().unwrap();
	let name: String = format!("{}{}", index, seed);
	let user = account(string_to_static_str(name), index, seed);
	T::Currency::make_free_balance_be(&user, balance_amount);
	T::Currency::issue(balance_amount);
	return user;
}

const PACKS: [PackService; 3] = [PackService::Basic, PackService::Medium, PackService::Max];

benchmarks! {
	join {
		let s in 0 .. 2;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u64);
	}: _(RawOrigin::Signed(caller), PACKS[s as usize])

	leave {
		let s in 0 .. 2;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u64);
		Pallet::<T>::join(RawOrigin::Signed(caller.clone()).into(), PACKS[s as usize]);
	}: _(RawOrigin::Signed(caller))

}

