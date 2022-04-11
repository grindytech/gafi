#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
};
use frame_system::pallet_prelude::*;
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	pool::{GafiPool, Level, Service, Ticket, TicketType},
	staking_pool::{Player, StakingPool},
};
pub use pallet::*;
use pallet_timestamp::{self as timestamp};

#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

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
		type WeightInfo: WeightInfo;
	}

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::storage]
	pub type Tickets<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Ticket<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn player_count)]
	pub type PlayerCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	pub type Services<T: Config> = StorageMap<_, Twox64Concat, Level, Service, ValueQuery>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub services: [(Level, Service); 3],
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				services: [
					(Level::Basic, Service::new(TicketType::Staking(Level::Basic))),
					(Level::Medium, Service::new(TicketType::Staking(Level::Medium))),
					(Level::Max, Service::new(TicketType::Staking(Level::Max))),
				],
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			for service in self.services {
				Services::<T>::insert(service.0, service.1);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		PlayerNotStake,
		StakeCountOverflow,
		DiscountNotCorrect,
		IntoBalanceFail,
	}

	impl<T: Config> GafiPool<T::AccountId> for Pallet<T> {
		fn join(sender: T::AccountId, level: Level) -> DispatchResult {
			let service = Services::<T>::get(level);
			let staking_amount = Self::u128_try_to_balance(service.value)?;
			<T as pallet::Config>::Currency::reserve(&sender, staking_amount)?;

			let new_player_count =
				Self::player_count().checked_add(1).ok_or(<Error<T>>::StakeCountOverflow)?;

			Self::stake_pool(sender, new_player_count, level);
			Ok(())
		}

		fn leave(sender: T::AccountId) -> DispatchResult {
			if let Some(player_level) = Self::get_player_level(sender.clone()) {
				let new_player_count =
					Self::player_count().checked_sub(1).ok_or(<Error<T>>::StakeCountOverflow)?;
				let service = Services::<T>::get(player_level);
				let staking_amount = Self::u128_try_to_balance(service.value)?;
				<T as pallet::Config>::Currency::unreserve(&sender, staking_amount);
				Self::unstake_pool(sender, new_player_count);
				Ok(())
			} else {
				Err(Error::<T>::PlayerNotStake.into())
			}
		}

		fn get_service(level: Level) -> Service {
			Services::<T>::get(level)
		}
	}

	impl<T: Config> Pallet<T> {
		fn stake_pool(sender: T::AccountId, new_player_count: u32, level: Level) {
			let _now = Self::moment_to_u128(<timestamp::Pallet<T>>::get());
			<PlayerCount<T>>::put(new_player_count);
			let ticket = Ticket {
				address: sender.clone(),
				join_time: _now,
				ticket_type: TicketType::Staking(level),
			};
			Tickets::<T>::insert(sender, ticket);
		}

		fn unstake_pool(sender: T::AccountId, new_player_count: u32) {
			<PlayerCount<T>>::put(new_player_count);
			Tickets::<T>::remove(sender);
		}

		pub fn moment_to_u128(input: T::Moment) -> u128 {
			sp_runtime::SaturatedConversion::saturated_into(input)
		}

		pub fn u128_try_to_balance(input: u128) -> Result<BalanceOf<T>, Error<T>> {
			match input.try_into().ok() {
				Some(val) => Ok(val),
				None => Err(<Error<T>>::IntoBalanceFail),
			}
		}

		fn get_player_level(player: T::AccountId) -> Option<Level> {
			match Tickets::<T>::get(player) {
				Some(ticket) => {
					if let TicketType::Staking(level) = ticket.ticket_type {
						Some(level)
					} else {
						None
					}
				},
				None => None,
			}
		}
	}
}
