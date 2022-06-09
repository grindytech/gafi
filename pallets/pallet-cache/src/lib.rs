// This file is part of Gafi Network.

// Copyright (C) 2021-2022 CryptoViet.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use frame_system::pallet_prelude::*;
use gafi_primitives::cache::Cache;
use pallet_timestamp::{self as timestamp};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Wrap data with the timestamp at the time when data insert into Cache
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub(super) struct WrapData<Data> {
		pub data: Data,
		pub timestamp: u128,
	}

	impl<Data> WrapData<Data> {
		fn new(data: Data, timestamp: u128) -> Self {
			WrapData { data, timestamp }
		}
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub(super) enum Flag {
		Left,
		Right,
	}

	impl Default for Flag {
		fn default() -> Self {
			Flag::Left
		}
	}

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_timestamp::Config {
		/// The overarching event type.
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Data contain the data that need to be storage to cache
		type Data: Parameter + MaxEncodedLen + Copy + TypeInfo;

		/// The Action is the name of action use to query
		type Action: Parameter + MaxEncodedLen + Clone + TypeInfo;

		#[pallet::constant]
		type CleanTime: Get<u128>;
	}

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		pub phantom: PhantomData<T>,
		pub phantom_i: PhantomData<I>,
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self {
				phantom: Default::default(),
				phantom_i: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>,  I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			let _now: u128 = <timestamp::Pallet<T>>::get()
				.try_into()
				.ok()
				.unwrap_or_default();
			<MarkTime<T, I>>::put(_now);
		}
	}

	//** STORAGE **//
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(_);

	/// Holding the flag(Left or Right) to support Cache in insert and clean
	#[pallet::type_value]
	pub(super) fn DefaultDataFlag() -> Flag {
		Flag::Left
	}
	#[pallet::storage]
	pub(super) type DataFlag<T: Config<I>,  I: 'static = ()> = StorageValue<_, Flag, ValueQuery, DefaultDataFlag>;

	/// Holding the data that insert in Cache by keys AccountId and Action
	#[pallet::storage]
	pub(super) type DataLeft<T: Config<I>,  I: 'static = ()> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::Action, WrapData<T::Data>>;

	/// Holding the data that insert in Cache by keys AccountId and Action
	#[pallet::storage]
	pub(super) type DataRight<T: Config<I>,  I: 'static = ()> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::Action, WrapData<T::Data>>;

	/// Holding the mark time clean cache
	/// The default value is at the time chain launched
	#[pallet::type_value]
	pub fn DefaultMarkTime<T: Config<I>, I: 'static>() -> u128 {
		<timestamp::Pallet<T>>::get().try_into().ok().unwrap()
	}
	#[pallet::storage]
	#[pallet::getter(fn mark_time)]
	pub type MarkTime<T: Config<I>,  I: 'static = ()> = StorageValue<_, u128, ValueQuery, DefaultMarkTime<T, I>>;

	//** HOOKS **//
	#[pallet::hooks]
	impl<T: Config<I>,  I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn on_finalize(_block_number: BlockNumberFor<T>) {
			let _now: u128 = Self::get_timestamp();
			if _now - Self::mark_time() >= T::CleanTime::get() {
				if DataFlag::<T, I>::get() == Flag::Left {
					<DataRight<T, I>>::remove_all(None);
					DataFlag::<T, I>::put(Flag::Right);
				} else {
					<DataLeft<T, I>>::remove_all(None);
					DataFlag::<T, I>::put(Flag::Left);
				}
				MarkTime::<T, I>::put(_now);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>,  I: 'static = ()> {}

	#[pallet::error]
	pub enum Error<T, I = ()> {}

	#[pallet::call]
	impl<T: Config<I>,  I: 'static> Pallet<T, I> {}

	impl<T: Config<I>,  I: 'static> Pallet<T, I> {
		pub fn get_timestamp() -> u128 {
			let _now: u128 = <timestamp::Pallet<T>>::get()
				.try_into()
				.ok()
				.unwrap_or_else(|| u128::default());
			_now
		}
	}

	impl<T: Config<I>,  I: 'static> Cache<T::AccountId, T::Action, T::Data> for Pallet<T, I> {

		/// Store data to cache by AccountId and action name
		///
		/// Parameters:
		/// - `id`: data owner
		/// - `action`: The action name
		///	- `data`: The data to store in the cache
		///
		/// Weight: `O(1)`
		fn insert(id: &T::AccountId, action: T::Action, data: T::Data) {
			let _now = Self::get_timestamp();
			let wrap_data = WrapData::new(data, _now);
			if DataFlag::<T, I>::get() == Flag::Left {
				DataLeft::<T, I>::insert(id, action, wrap_data);
			} else {
				DataRight::<T, I>::insert(id, action, wrap_data);
			}
		}

		/// Get valid data in cache by AccountId and action name
		///
		/// Parameters:
		/// - `id`: data owner
		///	- `action`: action name
		///
		/// Weight: `O(1)`
		fn get(id: &T::AccountId, action: T::Action) -> Option<T::Data> {
			let get_wrap_data = || -> Option<WrapData<T::Data>> {
				if let Some(data) = DataLeft::<T, I>::get(id, action.clone()) {
					return Some(data);
				} else if let Some(data) = DataRight::<T, I>::get(id, action.clone()) {
					return Some(data);
				}
				None
			};

			if let Some(wrap_data) = get_wrap_data() {
				let _now = Self::get_timestamp();
				if _now - wrap_data.timestamp < T::CleanTime::get() {
					return Some(wrap_data.data);
				}
			}
			None
		}
	}
}
