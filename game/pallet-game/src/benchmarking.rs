//! Benchmarking setup for pallet-pool

use super::*;
#[allow(unused)]
use crate::Pallet as PalletGame;
use crate::{Call, Config};
use enumflags2::{BitFlag, BitFlags};
use frame_benchmarking::{account, benchmarks, benchmarks_instance_pallet, Box};
use frame_support::{dispatch::UnfilteredDispatchable, traits::Currency};
use frame_system::RawOrigin;
use pallet_nfts::{CollectionSetting, CollectionSettings, ItemSettings, MintSettings};
use scale_info::prelude::{format, string::String};
use pallet_nfts::BenchmarkHelper;
use crate::pallet::BenchmarkHelper as GameBenchmarkHelper;

const UNIT: u128 = 1_000_000_000_000_000_000u128;
const MAX: u32 = 10_u32;

fn string_to_static_str(s: String) -> &'static str {
	Box::leak(s.into_boxed_str())
}

fn new_funded_account<T: Config<I>, I: 'static>(
	index: u32,
	seed: u32,
	amount: u128,
) -> T::AccountId {
	let balance_amount = amount.try_into().ok().unwrap();
	let name: String = format!("{}{}", index, seed);
	let user = account(string_to_static_str(name), index, seed);
	<T as pallet::Config<I>>::Currency::make_free_balance_be(&user, balance_amount);
	// <T as pallet::Config>::Currency::issue(balance_amount);
	return user
}

fn make_collection_config<T: Config<I>, I: 'static>(
	disable_settings: BitFlags<CollectionSetting>,
) -> CollectionConfigFor<T, I> {
	CollectionConfig {
		settings: CollectionSettings::from_disabled(disable_settings),
		max_supply: None,
		mint_settings: MintSettings::default(),
	}
}

fn default_collection_config<T: Config<I>, I: 'static>() -> CollectionConfigFor<T, I> {
	make_collection_config::<T, I>(CollectionSetting::empty())
}

fn default_item_config() -> ItemConfig {
	ItemConfig::default()
}

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::RuntimeEvent) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
	// compare to the last event record
	let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

benchmarks_instance_pallet! {

	create_game {
		let s in 0 .. MAX as u32;
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		let admin = new_funded_account::<T, I>(s + MAX, MAX, 1000_000_000u128 * UNIT);

		let call = Call::<T, I>::create_game { admin };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())? }
	verify {
		assert_last_event::<T, I>(Event::GameCreated { game: <T as pallet::Config<I>>::Helper::game(0) }.into());
	}


	create_game_collection {
		let s in 0 .. MAX as u32;
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		let admin = new_funded_account::<T, I>(s + MAX, MAX, 1000_000_000u128 * UNIT);

		let call = Call::<T, I>::create_game_collection { game: <T as pallet::Config<I>>::Helper::game(0), admin , config: default_collection_config::<T, I>() };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())? }
	verify {
		assert_last_event::<T, I>(Event::CollectionCreated { collection: <T as pallet_nfts::Config>::Helper::collection(0) }.into());
	}
}
