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
use aurora_primitives::{unit, currency::NativeToken::AUX};

#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::storage]
	pub type Players<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Player<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn player_count)]
	pub type PlayerCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	pub type StakeAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub stake_amount: BalanceOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			let stake_amount: u128 = 1000 * unit(AUX); 
			let into_balance = |fee: u128| -> BalanceOf<T> { fee.try_into().ok().unwrap() };
			Self {
				stake_amount: into_balance(stake_amount),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
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
		PlayerAlreadyStake,
		StakeCountOverflow,
	}
	
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		#[pallet::weight(1000)]
		pub fn stake(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(<Players::<T>>::get(sender.clone()) == None, <Error<T>>::PlayerAlreadyStake);

			let stake_amount = <StakeAmount<T>>::get();
			<T as pallet::Config>::Currency::reserve(&sender, stake_amount)?;
			
			let new_player_count =
			Self::player_count().checked_add(1).ok_or(<Error<T>>::StakeCountOverflow)?;

			Self::stake_pool(sender, new_player_count);
			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn unstake(origin: OriginFor<T>) -> DispatchResult {

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn stake_pool(sender: T::AccountId, new_player_count: u32) {

			let _now = Self::moment_to_u128(<timestamp::Pallet<T>>::get());

			let player = Player {
				address: sender.clone(),
				join_time: _now,
			};
			<PlayerCount<T>>::put(new_player_count);
			<Players<T>>::insert(sender, player);

		}

		pub fn moment_to_u128(input: T::Moment) -> u128 {
			sp_runtime::SaturatedConversion::saturated_into(input)
		}
	
	}
}
