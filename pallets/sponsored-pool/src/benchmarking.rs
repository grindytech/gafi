//! Benchmarking setup for pallet-pool

use super::*;
#[allow(unused)]
use crate::Pallet as SponsoredPool;
use crate::{Call, Config};
use frame_benchmarking::Box;
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use gafi_primitives::ticket::{TicketLevel, TicketType};
use scale_info::prelude::format;
use scale_info::prelude::string::String;
use sp_core::H160;
use sp_std::{str::FromStr, vec};

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

benchmarks! {
	create_pool {
		let s in 0 .. 10 as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		let targets = vec![
			H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap(),
		];
		let value = (1000_u128 * UNIT).try_into().ok().unwrap();
		let discount = 30_u8;
		let tx_limit = 100_u32;
	}: _(RawOrigin::Signed(caller), targets, value, discount, tx_limit)


	withdraw_pool {
		let s in 0 .. 10 as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		let targets = vec![
			H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap(),
		];
		let value: BalanceOf<T> = (1000_u128 * UNIT).try_into().ok().unwrap();
		let discount = 30_u8;
		let tx_limit = 100_u32;
		SponsoredPool::<T>::create_pool(RawOrigin::Signed(caller.clone()).into(), targets, value, discount, tx_limit);
		let pool_id: ID = *PoolOwned::<T>::get(caller.clone()).last().unwrap();
	}: _(RawOrigin::Signed(caller), pool_id)

	new_targets {
		let s in 0 .. 10 as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		let targets = vec![
			H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap(),
		];
		let value: BalanceOf<T> = (1000_u128 * UNIT).try_into().ok().unwrap();
		let discount = 30_u8;
		let tx_limit = 100_u32;
		SponsoredPool::<T>::create_pool(RawOrigin::Signed(caller.clone()).into(), targets, value, discount, tx_limit);
		let pool_id: ID = *PoolOwned::<T>::get(caller.clone()).last().unwrap();

		let targets = vec![
			H160::from_str("dAC17F958D2ee523a2206206994597C13D831ec7").unwrap(),
		];
	}: _(RawOrigin::Signed(caller), pool_id, targets)

	set_pool_name {
		let s in 0 .. 10 as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		let targets = vec![
			H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap(),
		];
		let value: BalanceOf<T> = (1000_u128 * UNIT).try_into().ok().unwrap();
		let discount = 30_u8;
		let tx_limit = 100_u32;
		SponsoredPool::<T>::create_pool(RawOrigin::Signed(caller.clone()).into(), targets, value, discount, tx_limit);
		let pool_id: ID = *PoolOwned::<T>::get(caller.clone()).last().unwrap();
	}: _(RawOrigin::Signed(caller), pool_id, b"Test pool".to_vec())

	clear_pool_name {
		let s in 0 .. 10 as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		let targets = vec![
			H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap(),
		];
		let value: BalanceOf<T> = (1000_u128 * UNIT).try_into().ok().unwrap();
		let discount = 30_u8;
		let tx_limit = 100_u32;
		SponsoredPool::<T>::create_pool(RawOrigin::Signed(caller.clone()).into(), targets, value, discount, tx_limit);
		let pool_id: ID = *PoolOwned::<T>::get(caller.clone()).last().unwrap();
		SponsoredPool::<T>::set_pool_name(RawOrigin::Signed(caller.clone()).into(), pool_id, b"Test pool".to_vec());
	}: _(RawOrigin::Signed(caller), pool_id)

	kill_pool_name {
		let s in 0 .. 10 as u32;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u128 * UNIT);
		let targets = vec![
			H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap(),
		];
		let value: BalanceOf<T> = (1000_u128 * UNIT).try_into().ok().unwrap();
		let discount = 30_u8;
		let tx_limit = 100_u32;
		SponsoredPool::<T>::create_pool(RawOrigin::Signed(caller.clone()).into(), targets, value, discount, tx_limit);
		let pool_id: ID = *PoolOwned::<T>::get(caller.clone()).last().unwrap();
		SponsoredPool::<T>::set_pool_name(RawOrigin::Signed(caller.clone()).into(), pool_id, b"Test pool".to_vec());
	}: _(RawOrigin::Root, pool_id)
}
