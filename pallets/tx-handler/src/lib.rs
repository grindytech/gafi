#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{print, traits::Hash},
		traits::{
			tokens::{ExistenceRequirement, WithdrawReasons},
			Currency, Randomness,
		},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_256;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		type GameRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// Errors.
	#[derive(PartialEq)]
	#[pallet::error]
	pub enum Error<T> {

	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		
	}

	// Storage


	#[pallet::call]
	impl<T: Config> Pallet<T> {
	}

	impl<T: Config> Pallet<T> {
		
	}
}
