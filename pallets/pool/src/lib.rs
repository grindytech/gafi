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
use frame_support::{pallet_prelude::*, traits::Currency, transactional};
use frame_system::pallet_prelude::*;
use gafi_primitives::{
	constant::ID,
	custom_services::CustomPool,
	pool::{MasterPool, PoolType, Service},
	system_services::SystemPool,
	ticket::TicketInfo,
	ticket::{PlayerTicket, TicketType},
};
use pallet_timestamp::{self as timestamp};

use crate::weights::WeightInfo;
use gafi_primitives::cache::Cache;
pub use pallet::*;
use sp_core::H160;
use sp_std::vec::Vec;

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
	use frame_support::Twox64Concat;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency mechanism.
		type Currency: Currency<Self::AccountId>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Add upfront pool
		type UpfrontPool: SystemPool<Self::AccountId>;

		/// Add Staking Pool
		type StakingPool: SystemPool<Self::AccountId>;

		/// Add Sponsored Pool
		type SponsoredPool: CustomPool<Self::AccountId>;

		#[pallet::constant]
		type MaxJoinedSponsoredPool: Get<u32>;

		#[pallet::constant]
		type TimeServiceStorage: Get<u128>;

		/// Add Cache
		type Cache: Cache<Self::AccountId, ID, TicketInfo>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// /// Holding all the tickets in the network
	// #[pallet::storage]
	// #[pallet::getter(fn tickets)]
	// pub type Tickets<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, TicketInfo>;

	/// Holding all the tickets in the network
	#[pallet::storage]
	#[pallet::getter(fn tickets)]
	pub type Tickets<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, ID, TicketInfo>;

	/// Holding the mark time to check if correct time to charge service fee
	/// The default value is at the time chain launched
	#[pallet::type_value]
	pub fn DefaultMarkTime<T: Config>() -> u128 {
		<timestamp::Pallet<T>>::get().try_into().ok().unwrap()
	}
	#[pallet::storage]
	#[pallet::getter(fn mark_time)]
	pub type MarkTime<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMarkTime<T>>;

	/// Honding the specific period of time to charge service fee
	/// The default value is 1 hours
	#[pallet::type_value]
	pub fn DefaultTimeService() -> u128 {
		// 1 hour
		3_600_000u128
	}
	#[pallet::storage]
	#[pallet::getter(fn time_service)]
	pub type TimeService<T: Config> = StorageValue<_, u128, ValueQuery, DefaultTimeService>;

	/// on_finalize following by steps:
	/// 1. renew tickets
	/// 2. Update new Marktime
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_block_number: BlockNumberFor<T>) {
			let _now: u128 = <timestamp::Pallet<T>>::get().try_into().ok().unwrap();
			if _now - Self::mark_time() >= Self::time_service() {
				Self::renew_tickets();
				MarkTime::<T>::put(_now);
			}
		}
	}

	//** Genesis Conguration **//
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
			<TimeService<T>>::put(<T as Config>::TimeServiceStorage::get());
			let _now: u128 = <timestamp::Pallet<T>>::get()
				.try_into()
				.ok()
				.unwrap_or_default();
			<MarkTime<T>>::put(_now);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Joined {
			sender: T::AccountId,
			pool_id: ID,
		},
		Leaved {
			sender: T::AccountId,
			ticket: TicketType,
		},
		LeavedAll {
			sender: T::AccountId,
			pool_type: PoolType,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyJoined,
		NotFoundInPool,
		TicketNotFound,
		ComingSoon,
		ExceedJoinedPool,
		PoolNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// join pool
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `ticket`: ticket type
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::join(50u32))]
		#[transactional]
		pub fn join(origin: OriginFor<T>, pool_id: ID) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				Self::is_joined_pool(sender.clone(), pool_id) == false,
				<Error<T>>::AlreadyJoined
			);

			let ticket_type = Self::get_ticket_type(pool_id)?;
			match ticket_type {
				TicketType::Upfront(_) => {
					T::UpfrontPool::join(sender.clone(), pool_id)?;
				}
				TicketType::Staking(_) => {
					T::StakingPool::join(sender.clone(), pool_id)?;
				}
				TicketType::Sponsored(_) => {
					let joined_sponsored_pool = Tickets::<T>::iter_prefix_values(sender.clone());
					let count_joined_pool = joined_sponsored_pool.count();

					ensure!(
						count_joined_pool <= T::MaxJoinedSponsoredPool::get() as usize,
						<Error<T>>::ExceedJoinedPool
					);

					T::SponsoredPool::join(sender.clone(), pool_id)?
				}
			}

			let ticket_info = Self::create_ticket(&sender, pool_id)?;

			Tickets::<T>::insert(sender.clone(), pool_id, ticket_info);

			Self::deposit_event(Event::<T>::Joined { sender, pool_id });
			Ok(())
		}

		/// leave pool
		///
		/// The origin must be Signed
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::leave(50u32))]
		#[transactional]
		pub fn leave(origin: OriginFor<T>, pool_id: ID) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			if let Some(ticket) = Tickets::<T>::get(sender.clone(), pool_id) {
				match ticket.ticket_type {
					TicketType::Upfront(_) => {
						T::UpfrontPool::leave(sender.clone())?
					}
					TicketType::Staking(_) => {
						T::StakingPool::leave(sender.clone())?
					}
					TicketType::Sponsored(_) => {
						T::SponsoredPool::leave(sender.clone())?
					}
				}
				T::Cache::insert(&sender, pool_id, ticket);
				Tickets::<T>::remove(sender.clone(), pool_id);
				Self::deposit_event(Event::<T>::Leaved {
					sender,
					ticket: ticket.ticket_type,
				});
				Ok(())
			} else {
				Err(Error::<T>::NotFoundInPool.into())
			}
		}

		/// Leave Pool
		///
		/// Leave all the pools that player joined
		///
		/// The origin must be Signed
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::leave_all(50u32))]
		#[transactional]
		pub fn leave_all(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			if let Ok(_) = T::UpfrontPool::leave(sender.clone()) {
				Self::deposit_event(Event::LeavedAll {
					sender: sender.clone(),
					pool_type: PoolType::Upfront,
				});
			} else if let Ok(_) = T::StakingPool::leave(sender.clone()) {
				Self::deposit_event(Event::LeavedAll {
					sender: sender.clone(),
					pool_type: PoolType::Staking,
				});
			} else if let Ok(_) = T::SponsoredPool::leave(sender.clone()) {
				Self::deposit_event(Event::LeavedAll {
					sender: sender.clone(),
					pool_type: PoolType::Sponsored,
				});
			}
			Tickets::<T>::remove_prefix(sender, None);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn create_ticket(
			sender: &T::AccountId,
			pool_id: ID,
		) -> Result<TicketInfo, Error<T>> {
			let ticket_type = Self::get_ticket_type(pool_id)?;
			if let Some(cache) = Self::get_cache(&sender, pool_id) {
				return Ok(TicketInfo {
					ticket_type,
					tickets: cache.tickets,
				});
			}

			let service = Self::get_ticket_service(ticket_type)?;

			Ok(TicketInfo {
				ticket_type,
				tickets: service.tx_limit,
			})
		}

		fn get_cache(sender: &T::AccountId, pool_id: ID) -> Option<TicketInfo> {
			if let Some(info) = T::Cache::get(&sender, pool_id) {
				return Some(info);
			}
			None
		}

		pub fn renew_tickets() {
			let _ = Tickets::<T>::iter().for_each(|player| {
				if let Some(ticket_info) = Tickets::<T>::get(player.0.clone(), player.1.clone()) {
					if let Some(service) = Self::get_service(player.1.clone()) {
						let new_ticket = ticket_info.renew_ticket(service.tx_limit);
						Tickets::<T>::insert(player.0, player.1, new_ticket);
					}
				}
			});
		}

		fn is_joined_pool(sender: T::AccountId, pool_id: ID) -> bool {
			let joined_pools = Tickets::<T>::iter_prefix_values(sender.clone());
			let mut is_joined = false;

			for joined_ticket in joined_pools {
				match joined_ticket.ticket_type {
					TicketType::Upfront(_) => is_joined = true,
					TicketType::Staking(_) => is_joined = true,
					TicketType::Sponsored(joined_pool_id) => {
						// We can join multiple sponsored pools so we must check equal id.
						if joined_pool_id == pool_id {
							is_joined = true;
						}
					}
				}
			}
			is_joined
		}

		fn get_ticket_service(ticket: TicketType) -> Result<Service, Error<T>> {
			match ticket {
				TicketType::Staking(pool_id) => {
					if let Some(service) = T::StakingPool::get_service(pool_id) {
						return Ok(service.service);
					}
				}
				TicketType::Upfront(pool_id) => {
					if let Some(service) = T::UpfrontPool::get_service(pool_id) {
						return Ok(service.service);
					}
				}
				TicketType::Sponsored(pool_id) => {
					if let Some(service) = T::SponsoredPool::get_service(pool_id) {
						return Ok(service.service);
					}
				}
			}

			Err(Error::<T>::PoolNotFound)
		}

		fn get_ticket_type(pool_id: ID) -> Result<TicketType, Error<T>> {
			if T::UpfrontPool::get_service(pool_id).is_some() {
				return Ok(TicketType::Upfront(pool_id));
			}
			if T::StakingPool::get_service(pool_id).is_some() {
				return Ok(TicketType::Staking(pool_id));
			}
			if T::SponsoredPool::get_service(pool_id).is_some() {
				return Ok(TicketType::Sponsored(pool_id));
			}
			Err(Error::<T>::PoolNotFound)
		}
	}

	impl<T: Config> PlayerTicket<T::AccountId> for Pallet<T> {
		fn use_ticket(player: T::AccountId, target: Option<H160>) -> Option<(TicketType, ID)> {
			let ticket_infos = Tickets::<T>::iter_prefix_values(player.clone());

			for ticket_info in ticket_infos {
				match ticket_info.ticket_type {
					TicketType::Upfront(pool_id) | TicketType::Staking(pool_id) => {
						if let Some(new_ticket_info) = ticket_info.withdraw_ticket() {
							Tickets::<T>::insert(
								player,
								pool_id,
								new_ticket_info,
							);
							return Some((
								new_ticket_info.ticket_type,
								pool_id,
							));
						}
					}
					TicketType::Sponsored(pool_id) => {
						if let Some(contract) = target {
							let targets = Self::get_targets(pool_id);
							if targets.contains(&contract) {
								if let Some(new_ticket_info) = ticket_info.withdraw_ticket() {
									Tickets::<T>::insert(player, pool_id, new_ticket_info);
									return Some((new_ticket_info.ticket_type, pool_id));
								}
							}
						}
					}
				}
			}
			None
		}

		fn get_service(pool_id: ID) -> Option<Service> {
			let upfront_service = T::UpfrontPool::get_service(pool_id);
			let staking_service = T::StakingPool::get_service(pool_id);
			let sponsored_service = T::SponsoredPool::get_service(pool_id);

			if upfront_service.is_some() {
				return Some(upfront_service.unwrap().service);
			}
			if staking_service.is_some() {
				return Some(staking_service.unwrap().service);
			}
			if sponsored_service.is_some() {
				return Some(sponsored_service.unwrap().service);
			}

			return None;
		}

		fn get_targets(pool_id: ID) -> Vec<H160> {
			match T::SponsoredPool::get_service(pool_id) {
				Some(service) => service.targets,
				None => [].to_vec(),
			}
		}
	}

	impl<T: Config> MasterPool<T::AccountId> for Pallet<T> {
		fn remove_player(player: &T::AccountId, pool_id: ID) {
			Tickets::<T>::remove(&player, pool_id)
		}

		fn get_timeservice() -> u128 {
			TimeService::<T>::get()
		}

		fn get_marktime() -> u128 {
			MarkTime::<T>::get()
		}
	}
}
