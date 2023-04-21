#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use sp_core::U256;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod features;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::traits::{
	tokens::nonfungibles_v2::{Create, Mutate, Transfer},
	Currency, Randomness, ReservableCurrency,
};
use frame_system::Config as SystemConfig;
use gafi_support::{common::constant::ID, game::GameSetting};
use sp_core::blake2_256;
use sp_runtime::traits::StaticLookup;

pub type DepositBalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use gafi_support::{common::types::BlockNumber};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(_);

	pub type BalanceOf<T, I = ()> =
		<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism, used for paying for reserves.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// generate random ID
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// The type used to identify a unique game
		type GameId: Member + Parameter + MaxEncodedLen + Copy;

		/// Max name length
		type MaxNameLength: Get<u32>;

		/// Min name length
		type MinNameLength: Get<u32>;

		/// Max Swapping Fee
		type MaxSwapFee: Get<u8>;
	}

	/// Store basic game info
	#[pallet::storage]
	pub(super) type Games<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::GameId, (T::AccountId, BoundedVec<u8, T::MaxNameLength>)>;

	#[pallet::storage]
	pub(super) type SwapFee<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::GameId, (u8, BlockNumber)>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		NotGameOwner,
		GameIdNotFound,
		NameTooLong,
		NameTooShort,
		SwapFeeTooHigh,
		SwapFeeNotFound,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {}

	
	
}
