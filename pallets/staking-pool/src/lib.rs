
// This file is part of Gafi Network.

// Copyright (C) 2021-2022 CryptoViet.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
};
use frame_system::pallet_prelude::*;
use gafi_primitives::{
	ticket::{TicketLevel, Ticket, TicketType, SystemTicket},
	system_services::{SystemPool, SystemService},
};
pub use pallet::*;
use pallet_timestamp::{self as timestamp};
use gu_convertor::{u128_try_to_balance};
use sp_runtime::Permill;

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
	use frame_support::{dispatch::DispatchResult};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	//** Storage **//

	/// Holding the number of maximum player can join the staking pool
	#[pallet::storage]
	pub type MaxPlayer<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Holding the tickets detail
	#[pallet::storage]
	pub type Tickets<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Ticket<T::AccountId>>;

	/// Player count
	#[pallet::storage]
	#[pallet::getter(fn player_count)]
	pub type PlayerCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Holding the services to serve to players, means service detail can change on runtime
	#[pallet::storage]
	#[pallet::getter(fn services)]
	pub type Services<T: Config> = StorageMap<_, Twox64Concat, TicketLevel, SystemService>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub services: [(TicketLevel, SystemService); 3],
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				services: [
					(TicketLevel::Basic, SystemService::new(100_u32, Permill::from_percent(30), 100000u128)),
					(TicketLevel::Medium, SystemService::new(100_u32, Permill::from_percent(50), 100000u128)),
					(TicketLevel::Advance,  SystemService::new(100_u32, Permill::from_percent(70), 100000u128)),
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
	pub enum Event<T: Config> {
		StakingNewMaxPlayer { new_max_player: u32 }
	}

	#[pallet::error]
	pub enum Error<T> {
		PlayerNotStake,
		StakeCountOverflow,
		IntoBalanceFail,
		LevelNotFound,
	}

	impl<T: Config> SystemPool<T::AccountId> for Pallet<T> {
			/// Join Staking Pool
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `level`: The level of ticket Basic - Medium - Advance
		///
		/// Weight: `O(1)`
		fn join(sender: T::AccountId, level: TicketLevel) -> DispatchResult {
			let service = Self::get_service_by_level(level)?;
			let staking_amount = u128_try_to_balance::<<T as pallet::Config>::Currency, T::AccountId>(service.value)?;
			<T as pallet::Config>::Currency::reserve(&sender, staking_amount)?;

			let new_player_count =
				Self::player_count().checked_add(1).ok_or(<Error<T>>::StakeCountOverflow)?;

			Self::stake_pool(sender, new_player_count, level);
			Ok(())
		}

		/// Leave Upfront Pool
		///
		/// The origin must be Signed
		///
		/// Weight: `O(1)`
		fn leave(sender: T::AccountId) -> DispatchResult {
			if let Some(level) = Self::get_player_level(sender.clone()) {
				let new_player_count =
					Self::player_count().checked_sub(1).ok_or(<Error<T>>::StakeCountOverflow)?;
				let service = Self::get_service_by_level(level)?;
				let staking_amount = u128_try_to_balance::<<T as pallet::Config>::Currency, T::AccountId>(service.value)?;
				<T as pallet::Config>::Currency::unreserve(&sender, staking_amount);
				Self::unstake_pool(sender, new_player_count);
				Ok(())
			} else {
				Err(Error::<T>::PlayerNotStake.into())
			}
		}

		fn get_service(level: TicketLevel) -> Option<SystemService> {
			Services::<T>::get(level)
		}
	}

	#[pallet::call]
	impl <T: Config> Pallet<T> {
		/// Set MaxPlayer
		///
		/// The root must be Signed
		///
		/// Parameters:
		/// - `max_player`: new value of MaxPlayer
		///
		/// Weight: `O(1)`
		#[pallet::weight((
			0,
			DispatchClass::Normal,
			Pays::No
		))]
		pub fn set_max_player(origin: OriginFor<T>, new_max_player: u32) -> DispatchResult {
			ensure_root(origin)?;
			MaxPlayer::<T>::put(new_max_player);
			Self::deposit_event(Event::<T>::StakingNewMaxPlayer{new_max_player});
			Ok(())
		}
	}


	impl<T: Config> Pallet<T> {
		fn stake_pool(sender: T::AccountId, new_player_count: u32, level: TicketLevel) {
			let _now = Self::moment_to_u128(<timestamp::Pallet<T>>::get());
			<PlayerCount<T>>::put(new_player_count);
			let ticket = Ticket {
				address: sender.clone(),
				join_time: _now,
				ticket_type: TicketType::System(SystemTicket::Staking(level)),
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

		fn get_player_level(player: T::AccountId) -> Option<TicketLevel> {
			match Tickets::<T>::get(player) {
				Some(ticket) => {
					if let TicketType::System(SystemTicket::Staking(level)) = ticket.ticket_type {
						Some(level)
					} else {
						None
					}
				},
				None => None,
			}
		}

		fn get_service_by_level(level: TicketLevel) -> Result<SystemService, Error<T>> {
			match Services::<T>::get(level) {
				Some(service) => Ok(service),
				None => Err(<Error<T>>::LevelNotFound),
			}
		}
	}
}
