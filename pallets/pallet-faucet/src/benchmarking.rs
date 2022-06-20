//! Benchmarking setup for pallet-faucet

use super::*;
use frame_benchmarking::{Box, benchmarks, whitelisted_caller, account};
use frame_system::RawOrigin;
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
	<T as pallet::Config>::Currency::make_free_balance_be(&user, balance_amount);
	<T as pallet::Config>::Currency::issue(balance_amount);
	return user;
}

benchmarks!{
	faucet {
		let b in 0 .. 1;
		let caller = whitelisted_caller();
	}: _(RawOrigin::Signed(caller))

	donate {
		let b in 1 .. 10 ;
		let caller = new_funded_account::<T>(b,b, 1000_u128 * UNIT);
	}: _(RawOrigin::Signed(caller), (10_u128 * UNIT).try_into().ok().unwrap())
}
