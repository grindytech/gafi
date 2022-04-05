#![cfg_attr(not(feature = "std"), no_std)]
use aurora_primitives::{currency::NativeToken::AUX, unit};
use frame_support::{
	dispatch::{DispatchResult, Vec},
	pallet_prelude::*,
	traits::{
		tokens::{ExistenceRequirement, WithdrawReasons},
		Currency, ReservableCurrency,
	},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
pub use pallet::*;
use pallet_timestamp::{self as timestamp};
use sp_runtime::Perbill;

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
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Player<AccountId> {
	pub address: AccountId,
	pub join_time: u128,
}

pub trait StakingPool<AccountId> {
	fn is_staking_pool(player: &AccountId) -> Option<Player<AccountId>>;

	fn staking_pool_discount() -> u8;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};

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

	#[pallet::storage]
	pub type Discount<T: Config> = StorageValue<_, u8, ValueQuery>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub staking_amount: BalanceOf<T>,
		pub staking_discount: u8,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			let staking_amount: u128 = 1000 * unit(AUX);
			let into_balance = |fee: u128| -> BalanceOf<T> { fee.try_into().ok().unwrap() };
			Self { staking_amount: into_balance(staking_amount), staking_discount: 50 }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<StakeAmount<T>>::put(self.staking_amount);
			<Discount<T>>::put(self.staking_discount);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		PlayerAlreadyStake,
		PlayerNotStake,
		StakeCountOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1000)]
		pub fn stake(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(<Players::<T>>::get(sender.clone()) == None, <Error<T>>::PlayerAlreadyStake);

			let staking_amount = <StakeAmount<T>>::get();
			<T as pallet::Config>::Currency::reserve(&sender, staking_amount)?;

			let new_player_count =
				Self::player_count().checked_add(1).ok_or(<Error<T>>::StakeCountOverflow)?;

			Self::stake_pool(sender, new_player_count);
			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn unstake(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(<Players::<T>>::get(sender.clone()) != None, <Error<T>>::PlayerNotStake);
			let staking_amount = <StakeAmount<T>>::get();
			let new_player_count =
				Self::player_count().checked_sub(1).ok_or(<Error<T>>::StakeCountOverflow)?;

			<T as pallet::Config>::Currency::unreserve(&sender, staking_amount);
			Self::unstake_pool(sender, new_player_count);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn stake_pool(sender: T::AccountId, new_player_count: u32) {
			let _now = Self::moment_to_u128(<timestamp::Pallet<T>>::get());

			let player = Player { address: sender.clone(), join_time: _now };
			<PlayerCount<T>>::put(new_player_count);
			<Players<T>>::insert(sender, player);
		}

		fn unstake_pool(sender: T::AccountId, new_player_count: u32) {
			<PlayerCount<T>>::put(new_player_count);
			<Players<T>>::remove(sender);
		}

		pub fn moment_to_u128(input: T::Moment) -> u128 {
			sp_runtime::SaturatedConversion::saturated_into(input)
		}
	}

	impl<T: Config> StakingPool<T::AccountId> for Pallet<T> {
		fn is_staking_pool(player: &T::AccountId) -> Option<Player<T::AccountId>> {
			Players::<T>::get(player)
		}

		fn staking_pool_discount() -> u8 {
			Discount::<T>::get()
		}
	}
}
