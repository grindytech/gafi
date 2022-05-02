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

	//** STORAGE **//
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type DataFlag<T: Config> = StorageValue<_, Flag, ValueQuery>;

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
			let _now: u128 = <timestamp::Pallet<T>>::get().try_into().ok().unwrap();
			if _now - Self::mark_time() >= Self::clean_time() {}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T: Config> Cache<T::AccountId, T::Data> for Pallet<T> {
		fn insert(id: T::AccountId, data: T::Data) {}

		fn get(id: T::AccountId) -> Option<T::Data> {
			todo!()
		}
	}
}
