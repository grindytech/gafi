#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use pallet_nfts::{ItemConfig, ItemSettings};
use sp_core::U256;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod mutable;

use frame_support::traits::{
	tokens::nonfungibles_v2::{Mutate, Transfer},
	Currency,
};
use frame_system::Config as SystemConfig;

pub type DepositBalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_nfts::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// pallet_nfts
		type Nfts: Mutate<Self::AccountId, ItemConfig> + Transfer<Self::AccountId>;

		/// The currency mechanism, used for paying for reserves.
		type Currency: frame_support::traits::ReservableCurrency<Self::AccountId>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	pub(super) type NftBalances<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::AccountId, (<T as pallet_nfts::Config>::ItemId, u32)>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {


	}
}
