#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use pallet_nfts::ItemSettings;
use sp_core::U256;
use pallet_nfts::ItemConfig;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod common;

use frame_system::Config as SystemConfig;
use frame_support::traits::{Currency, tokens::nonfungible_v2::{Mutate, Transfer}};

pub type DepositBalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

pub type DepositPerByte<T, I = ()> = <T as pallet::Config<I>>::DepositPerByte;

pub type StringLimit<T, I = ()> = <T as pallet::Config<I>>::StringLimit;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use common::GameNFT;
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
		type NFTs: Mutate<Self::AccountId, ItemConfig> + Transfer<Self::AccountId>;

		/// The additional funds that must be reserved for the number of bytes store in metadata,
		/// either "normal" metadata or attribute metadata.
		#[pallet::constant]
		type DepositPerByte: Get<DepositBalanceOf<Self, I>>;

		/// The currency mechanism, used for paying for reserves.
		type Currency: frame_support::traits::ReservableCurrency<Self::AccountId>;

		/// The maximum length of data stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	pub(super) type Something<T: Config<I>, I: 'static = ()> = StorageValue<_, u32>;

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
	impl<T: Config<I>, I: 'static> Pallet<T, I> {}

	impl<T: Config<I>, I: 'static> GameNFT<DepositPerByte<T, I>, StringLimit<T, I>, T::AccountId>
		for Pallet<T, I>
	{
		fn set_upgrade() -> Result<(), ()> {
			todo!()
		}

		fn upgrade(
			token_id: U256,
			address: T::AccountId,
			upgrade_data: common::UpgradeData<DepositPerByte<T, I>, StringLimit<T, I>>,
		) -> Result<(), ()> {

			todo!()
		}

		fn approve_upgrade(token_id: U256, address: T::AccountId) -> Result<(), ()> {
			todo!()
		}

		fn allow_combine(collection_id: u32) -> Result<(), ()> {
			todo!()
		}

		fn combine(token_id: U256, address: T::AccountId) -> Result<(), ()> {
			todo!()
		}
	}
}
