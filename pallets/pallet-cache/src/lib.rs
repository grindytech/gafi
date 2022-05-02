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

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub(super) struct WrapData<Data> {
		pub data: Data,
		pub timestamp: u128,
	}

	impl<Data> WrapData<Data> {
		fn new(data: Data, now: u128) -> Self {
			WrapData {
				data,
				timestamp: now,
			}
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
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		type Data: Parameter + MaxEncodedLen + Copy + TypeInfo;

		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub clean_time: u128,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				clean_time: 3_600_000u128,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<CleanTime<T>>::put(self.clean_time);
			let _now: u128 = <timestamp::Pallet<T>>::get()
				.try_into()
				.ok()
				.unwrap_or_default();
			<MarkTime<T>>::put(_now);
		}
	}

	//** STORAGE **//
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::type_value]
	pub(super) fn DefaultDataFlag() -> Flag {
		Flag::Left
	}
	#[pallet::storage]
	pub(super) type DataFlag<T: Config> = StorageValue<_, Flag, ValueQuery, DefaultDataFlag>;

	#[pallet::storage]
	pub(super) type DataLeft<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, WrapData<T::Data>>;
	#[pallet::storage]
	pub(super) type DataRight<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, WrapData<T::Data>>;

	/// Holding the mark time to check if correct time to charge service fee
	/// The default value is at the time chain launched
	#[pallet::type_value]
	pub fn DefaultMarkTime<T: Config>() -> u128 {
		<timestamp::Pallet<T>>::get().try_into().ok().unwrap()
	}
	#[pallet::storage]
	#[pallet::getter(fn mark_time)]
	pub type MarkTime<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMarkTime<T>>;

	/// Honding the specific period of time to clean data
	/// The default value is 1 hours
	#[pallet::type_value]
	pub fn DefaultCleanTime() -> u128 {
		// 1 hour
		3_600_000u128
	}
	#[pallet::storage]
	#[pallet::getter(fn clean_time)]
	pub type CleanTime<T: Config> = StorageValue<_, u128, ValueQuery, DefaultCleanTime>;

	//** HOOKS **//
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_block_number: BlockNumberFor<T>) {
			let _now: u128 = Self::get_timestamp();
			if _now - Self::mark_time() >= Self::clean_time() {
				if DataFlag::<T>::get() == Flag::Left {
					<DataRight<T>>::remove_all(None);
					DataFlag::<T>::put(Flag::Right);
				} else {
					<DataLeft<T>>::remove_all(None);
					DataFlag::<T>::put(Flag::Left);
				}
				MarkTime::<T>::put(_now);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T: Config> Pallet<T> {
		pub fn get_timestamp() -> u128 {
			let _now: u128 = <timestamp::Pallet<T>>::get()
				.try_into()
				.ok()
				.unwrap_or_else(|| u128::default());
			_now
		}
	}

	impl<T: Config> Cache<T::AccountId, T::Data> for Pallet<T> {
		fn insert(id: T::AccountId, data: T::Data) {
			let _now = Self::get_timestamp();
			let wrap_data = WrapData::new(data, _now);
			if DataFlag::<T>::get() == Flag::Left {
				DataLeft::<T>::insert(id, wrap_data);
			} else {
				DataRight::<T>::insert(id, wrap_data);
			}
		}

		fn get(id: T::AccountId) -> Option<T::Data> {
			let get_wrap_data = || -> Option<WrapData<T::Data>>{
				if let Some(data) = DataLeft::<T>::get(id.clone()) {
					return Some(data);
				} else if let Some(data) = DataRight::<T>::get(id) {
					return Some(data);
				}
				None
			};

			if let Some(wrap_data) = get_wrap_data() {
				let _now = Self::get_timestamp();
				if _now - wrap_data.timestamp < CleanTime::<T>::get() {
					return Some(wrap_data.data);
				}
			}
			None
		}
	}
}
