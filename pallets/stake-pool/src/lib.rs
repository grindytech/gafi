#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
use frame_support::{
	dispatch::{DispatchResult, Vec},
	pallet_prelude::*,
	traits::{
		tokens::{ExistenceRequirement, WithdrawReasons},
		Currency,
		ReservableCurrency
	},
};
use frame_system::pallet_prelude::*;
use pallet_timestamp::{self as timestamp};
use sp_runtime::{Perbill};
pub use pallet::*;
use frame_support::serde::{Deserialize, Serialize};
use aurora_primitives::{unit, currency::NativeToken::AUX};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

// Struct, Enum
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Player<AccountId> {
    pub address: AccountId,
    pub join_time: u128,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, dispatch::DispatchResult};
	
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	#[pallet::storage]
	pub type Players<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Player<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn player_count)]
	pub type PlayerCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	pub type StakeAmount<T: Config> = StorageValue<_, u128, ValueQuery>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub stake_amount: u128,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			let base_stake: u128 = 100 * unit(AUX);
			Self {
				stake_amount: base_stake,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<StakeAmount<T>>::put(self.stake_amount);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {

	}
	
	#[pallet::error]
	pub enum Error<T> {
	}
	
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		#[pallet::weight(1000)]
		pub fn stake(origin: OriginFor<T>) -> DispatchResult {
	
			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn unstake(origin: OriginFor<T>) -> DispatchResult {

			Ok(())
		}
	}
}
