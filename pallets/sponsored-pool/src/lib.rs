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

use crate::weights::WeightInfo;
use frame_support::pallet_prelude::*;
use frame_support::traits::{
	fungible::Inspect, Currency, ExistenceRequirement, Randomness, ReservableCurrency,
};
use frame_system::pallet_prelude::*;
pub use gafi_primitives::{
	constant::ID,
	name::Name,
	pool::{Level, Service, StaticPool, StaticService},
};
pub use pallet::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H160;
use sp_io::hashing::blake2_256;
use sp_std::vec::Vec;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
struct SponsoredPool<AccountId> {
	pub id: ID,
	pub owner: AccountId,
	pub value: u128,
	pub discount: u8,
	pub tx_limit: u32,
}

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

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub struct NewPool<AccountId> {
		pub id: ID,
		pub account: AccountId,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// To make the random pool id
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// Manage pool name
		type PoolName: Name<Self::AccountId>;

		/// The maximum number of pool that sponsor can create
		#[pallet::constant]
		type MaxPoolOwned: Get<u32>;

		/// The maximum number of contract address can added to the pool
		#[pallet::constant]
		type MaxPoolTarget: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	//** Storages **//
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Holding the all the pool data
	#[pallet::storage]
	pub(super) type Pools<T: Config> = StorageMap<_, Twox64Concat, ID, SponsoredPool<T::AccountId>>;

	/// Holding the pool owned
	#[pallet::storage]
	#[pallet::getter(fn pool_owned)]
	pub(super) type PoolOwned<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<ID, T::MaxPoolOwned>, ValueQuery>;

	/// Holding the contract addresses
	#[pallet::storage]
	pub(super) type Targets<T: Config> =
		StorageMap<_, Twox64Concat, ID, BoundedVec<H160, T::MaxPoolTarget>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreatedPool { id: ID },
		Withdrew { id: ID },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Generate the pool id that duplicated
		PoolIdExisted,
		/// Can not convert u128 to balance
		ConvertBalanceFail,
		/// Can not convert pool id to account
		IntoAccountFail,
		IntoU32Fail,
		/// Origin not the owner
		NotTheOwner,
		PoolNotExist,
		ExceedMaxPoolOwned,
		ExceedPoolTarget,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Create Pool
		///
		/// Create new pool and deposit amount of `value` to the pool,
		/// the origin must be Signed
		///
		/// Parameters:
		/// - `targets`: smart-contract addresses
		/// - `value`: the amount token deposit to the pool
		/// - `discount`: transaction fee discount
		/// - `tx_limit`: the number of discounted transaction per period of time
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_pool(50u32))]
		pub fn create_pool(
			origin: OriginFor<T>,
			targets: Vec<H160>,
			value: BalanceOf<T>,
			discount: u8,
			tx_limit: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let pool_config = Self::new_pool()?;
			ensure!(
				Pools::<T>::get(pool_config.id) == None,
				<Error<T>>::PoolIdExisted
			);
			ensure!(
				T::Currency::free_balance(&sender) > value,
				pallet_balances::Error::<T>::InsufficientBalance
			);
			ensure! {
				Self::usize_try_to_u32(targets.len())? <= T::MaxPoolTarget::get(),
				<Error<T>>::ExceedPoolTarget
			}

			let new_pool = SponsoredPool {
				id: pool_config.id,
				owner: sender.clone(),
				value: Self::balance_try_to_u128(value)?,
				discount,
				tx_limit,
			};

			<T as pallet::Config>::Currency::transfer(
				&sender,
				&pool_config.account,
				value,
				ExistenceRequirement::KeepAlive,
			)?;

			PoolOwned::<T>::try_mutate(&sender, |pool_vec| pool_vec.try_push(pool_config.id))
				.map_err(|_| <Error<T>>::ExceedMaxPoolOwned)?;
			Targets::<T>::try_mutate(pool_config.id, |target_vec| {
				for target in targets {
					if let Ok(_) = target_vec.try_push(target) {
					} else {
						return Err(());
					}
				}
				Ok(())
			})
			.map_err(|_| <Error<T>>::ExceedMaxPoolOwned)?;

			Pools::<T>::insert(pool_config.id, new_pool);

			Self::deposit_event(Event::CreatedPool { id: pool_config.id });
			Ok(())
		}

		/// Withdraw Pool
		///
		/// withdraw all the balances remain in the pool and destroy the pool,
		/// the origin as the owner of the pool must be Signed
		///
		/// Parameters:
		/// - `pool_id`: the id of the pool
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_pool(50u32))]
		pub fn withdraw_pool(origin: OriginFor<T>, pool_id: ID) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(Pools::<T>::get(pool_id) != None, <Error<T>>::PoolNotExist);
			ensure!(
				Self::is_pool_owner(&pool_id, &sender)?,
				<Error<T>>::NotTheOwner
			);
			let pool = Self::into_account(pool_id)?;
			Self::transfer_all(&pool, &sender, false)?;
			PoolOwned::<T>::try_mutate(&sender, |pool_owned| {
				if let Some(ind) = pool_owned.iter().position(|&id| id == pool_id) {
					pool_owned.swap_remove(ind);
					return Ok(());
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::PoolNotExist)?;
			Pools::<T>::remove(pool_id);
			Targets::<T>::remove(pool_id);
			Self::deposit_event(Event::Withdrew { id: pool_id });
			Ok(())
		}

		/// New Targets
		///
		/// change the contract addresses by replace old addresses with the new one
		/// the origin as the owner of the pool must be Signed
		///
		/// Parameters:
		/// - `pool_id`: the id of the pool
		/// - `targets`: new smart-contract addresses
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::new_targets(50u32))]
		pub fn new_targets(
			origin: OriginFor<T>,
			pool_id: ID,
			targets: Vec<H160>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				Self::is_pool_owner(&pool_id, &sender)?,
				<Error<T>>::NotTheOwner
			);
			ensure!(
				Self::usize_try_to_u32(targets.len())? < T::MaxPoolTarget::get(),
				<Error<T>>::ExceedPoolTarget
			);

			Targets::<T>::insert(pool_id, BoundedVec::default());
			Targets::<T>::try_mutate(&pool_id, |target_vec| {
				for target in targets {
					if let Ok(_) = target_vec.try_push(target) {
					} else {
						return Err(());
					}
				}
				Ok(())
			})
			.map_err(|_| <Error<T>>::ExceedPoolTarget)?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_pool_name(
			origin: OriginFor<T>,
			pool_id: ID,
			name: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			ensure!(Pools::<T>::get(pool_id) != None, <Error<T>>::PoolNotExist);
			ensure!(
				Self::is_pool_owner(&pool_id, &sender)?,
				<Error<T>>::NotTheOwner
			);

			T::PoolName::set_name(sender, pool_id, name)?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn clear_pool_name(
			origin: OriginFor<T>,
			pool_id: ID,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			ensure!(Pools::<T>::get(pool_id) != None, <Error<T>>::PoolNotExist);
			ensure!(
				Self::is_pool_owner(&pool_id, &sender)?,
				<Error<T>>::NotTheOwner
			);

			T::PoolName::clear_name(sender, pool_id)?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn kill_pool_name(
			origin: OriginFor<T>,
			pool_id: ID,
		) -> DispatchResult {
			ensure_root(origin.clone())?;

			match Pools::<T>::get(pool_id) {
				None => Err(<Error<T>>::PoolNotExist.into()),
				Some(pool) => Ok(T::PoolName::kill_name(pool.owner, pool_id)?),
			}
		}

	}

	impl<T: Config> Pallet<T> {
		fn gen_id() -> Result<ID, Error<T>> {
			let payload = (
				T::Randomness::random(&b""[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			Ok(payload.using_encoded(blake2_256))
		}

		pub(super) fn new_pool() -> Result<NewPool<T::AccountId>, Error<T>> {
			let id = Self::gen_id()?;
			match T::AccountId::decode(&mut &id[..]) {
				Ok(account) => Ok(NewPool::<T::AccountId> { id, account }),
				Err(_) => Err(<Error<T>>::IntoAccountFail),
			}
		}

		fn into_account(id: ID) -> Result<T::AccountId, Error<T>> {
			match T::AccountId::decode(&mut &id[..]) {
				Ok(account) => Ok(account),
				Err(_) => Err(<Error<T>>::IntoAccountFail),
			}
		}

		fn usize_try_to_u32(input: usize) -> Result<u32, Error<T>> {
			match input.try_into().ok() {
				Some(val) => Ok(val),
				None => Err(<Error<T>>::IntoU32Fail),
			}
		}

		fn balance_try_to_u128(input: BalanceOf<T>) -> Result<u128, Error<T>> {
			match input.try_into().ok() {
				Some(val) => Ok(val),
				None => Err(<Error<T>>::ConvertBalanceFail),
			}
		}

		fn transfer_all(
			from: &T::AccountId,
			to: &T::AccountId,
			keep_alive: bool,
		) -> DispatchResult {
			let reducible_balance: u128 =
				pallet_balances::pallet::Pallet::<T>::reducible_balance(from, keep_alive)
					.try_into()
					.ok()
					.unwrap();
			let existence = if keep_alive {
				ExistenceRequirement::KeepAlive
			} else {
				ExistenceRequirement::AllowDeath
			};
			<T as pallet::Config>::Currency::transfer(
				from,
				to,
				reducible_balance.try_into().ok().unwrap(),
				existence,
			)
		}

		fn is_pool_owner(pool_id: &ID, owner: &T::AccountId) -> Result<bool, Error<T>> {
			match Pools::<T>::get(pool_id) {
				Some(pool) => Ok(pool.owner == *owner),
				None => Err(<Error<T>>::PoolNotExist),
			}
		}
	}

	impl<T: Config> StaticPool<T::AccountId> for Pallet<T> {
		fn join(_sender: T::AccountId, _pool_id: ID) -> DispatchResult {
			Ok(())
		}
		fn leave(_sender: T::AccountId) -> DispatchResult {
			Ok(())
		}

		fn get_service(pool_id: ID) -> Option<StaticService<T::AccountId>> {
			if let Some(pool) = Pools::<T>::get(pool_id) {
				let targets = Targets::<T>::get(pool_id);
				return Some(StaticService::new(
					targets.to_vec(),
					pool.tx_limit,
					pool.discount,
					pool.owner,
				));
			}
			None
		}
	}
}
