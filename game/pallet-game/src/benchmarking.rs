//! Benchmarking setup for pallet-pool

use super::*;
#[allow(unused)]
use crate::Pallet as PalletGame;
use crate::{Call, Config};
use frame_benchmarking::Box;
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use scale_info::prelude::format;
use scale_info::prelude::string::String;

const UNIT: u128 = 1_000_000_000_000_000_000u128;
const MAX: u32 = 10_u32;

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

benchmarks! {
	create_game {
		let s in 0 .. MAX as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		let admin = new_funded_account::<T>(s + MAX, MAX, 1000_000_000u128 * UNIT);
	
	}: _(RawOrigin::Signed(caller), admin)
}