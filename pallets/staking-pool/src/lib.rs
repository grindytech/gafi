// This file is part of Gafi Network.

// Copyright (C) 2021-2022 Grindy Technologies.
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
	transactional,
};
use frame_system::pallet_prelude::*;
use gafi_primitives::{
	constant::ID,
	system_services::{ SystemDefaultServices, SystemPool, SystemService},
	ticket::{ Ticket, TicketType},
};
use gu_convertor::u128_try_to_balance;
pub use pallet::*;
use pallet_timestamp::{self as timestamp};

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
	use gafi_primitives::players::PlayersTime;
	use super::*;
	use frame_support::dispatch::DispatchResult;

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

		type StakingServices: SystemDefaultServices;

		type Players: PlayersTime<Self::AccountId>;
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
	pub type Services<T: Config> = StorageMap<_, Twox64Concat, ID, SystemService>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			for service in <T as Config>::StakingServices::get_default_services() {
				Services::<T>::insert(service.0, service.1);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		StakingNewMaxPlayer { new_max_player: u32 },
	}

	#[pallet::error]
	pub enum Error<T> {
		PlayerNotStake,
		StakeCountOverflow,
		IntoBalanceFail,
		LevelNotFound,
		PoolNotFound,
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
		#[transactional]
		fn join(sender: T::AccountId, pool_id: ID) -> DispatchResult {
			let service = Self::get_pool_by_id(pool_id)?;
			let staking_amount = u128_try_to_balance::<
				<T as pallet::Config>::Currency,
				T::AccountId,
			>(service.value)?;
			<T as pallet::Config>::Currency::reserve(&sender, staking_amount)?;

			let new_player_count = Self::player_count()
				.checked_add(1)
				.ok_or(<Error<T>>::StakeCountOverflow)?;

			Self::stake_pool(sender, pool_id, new_player_count)?;
			Ok(())
		}

		/// Leave Upfront Pool
		///
		/// The origin must be Signed
		///
		/// Weight: `O(1)`
		#[transactional]
		fn leave(sender: T::AccountId) -> DispatchResult {
			if let Some(ticket) = Tickets::<T>::get(&sender) {
				let new_player_count = Self::player_count()
					.checked_sub(1)
					.ok_or(<Error<T>>::StakeCountOverflow)?;

				let join_time = ticket.join_time;
				let _now = Self::moment_to_u128(<timestamp::Pallet<T>>::get());

				T::Players::add_time_joined_staking(sender.clone(), _now.saturating_sub(join_time));

				if let TicketType::Staking(pool_id) = ticket.ticket_type {
					let service = Self::get_pool_by_id(pool_id)?;
					let staking_amount = u128_try_to_balance::<
						<T as pallet::Config>::Currency,
						T::AccountId,
					>(service.value)?;
					<T as pallet::Config>::Currency::unreserve(&sender, staking_amount);
					Self::unstake_pool(sender, new_player_count);
					return Ok(());
				}
			}
			return Err(Error::<T>::PlayerNotStake.into());
		}

		fn get_service(pool_id: ID) -> Option<SystemService> {
			Services::<T>::get(pool_id)
		}

		fn get_ticket(sender: T::AccountId) -> Option<Ticket<T::AccountId>> {
			Tickets::<T>::get(sender.clone())
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
			Self::deposit_event(Event::<T>::StakingNewMaxPlayer { new_max_player });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn stake_pool(
			sender: T::AccountId,
			pool_id: ID,
			new_player_count: u32,
		) -> Result<(), Error<T>> {
			let _now = Self::moment_to_u128(<timestamp::Pallet<T>>::get());
			let pool: SystemService = Self::get_pool_by_id(pool_id)?;
			<PlayerCount<T>>::put(new_player_count);
			let ticket = Ticket {
				address: sender.clone(),
				join_time: _now,
				ticket_type: TicketType::Staking(pool_id),
			};
			Tickets::<T>::insert(sender, ticket);
			Ok(())
		}

		fn unstake_pool(sender: T::AccountId, new_player_count: u32) {
			<PlayerCount<T>>::put(new_player_count);
			Tickets::<T>::remove(sender);
		}

		pub fn moment_to_u128(input: T::Moment) -> u128 {
			sp_runtime::SaturatedConversion::saturated_into(input)
		}


		fn get_pool_by_id(pool_id: ID) -> Result<SystemService, Error<T>> {
			match Services::<T>::get(pool_id) {
				Some(service) => Ok(service),
				None => Err(<Error<T>>::PoolNotFound),
			}
		}
	}
}
