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

use crate::weights::WeightInfo;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency},
	transactional,
};
use frame_system::pallet_prelude::*;
pub use gafi_primitives::{
	constant::ID,
	custom_services::{CustomPool, CustomService},
	name::Name,
	pool::Service,
	whitelist::IWhitelist,
};
use gu_convertor::{balance_try_to_u128, into_account};
use gu_currency::transfer_all;
pub use pallet::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H160;
use sp_io::hashing::blake2_256;
use sp_runtime::Permill;
use sp_std::vec::Vec;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct FundingPool<AccountId> {
	pub id: ID,
	pub owner: AccountId,
	pub value: u128,
	pub discount: Permill,
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
				type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// To make the random pool id
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// Manage pool name
		type PoolName: Name<Self::AccountId>;

		/// The minimum balance owner have to deposit when creating the pool
		#[pallet::constant]
		type MinPoolBalance: Get<u128>;

		/// The minimum discount percent when creating the pool
		#[pallet::constant]
		type MinDiscountPercent: Get<Permill>;

		///The maximum discount percent when creating the pool
		#[pallet::constant]
		type MaxDiscountPercent: Get<Permill>;

		/// The minimum tx limit when creating the pool
		#[pallet::constant]
		type MinTxLimit: Get<u32>;

		///The maximum tx limit when creating the pool
		#[pallet::constant]
		type MaxTxLimit: Get<u32>;

		/// The maximum number of pool that sponsor can create
		#[pallet::constant]
		type MaxPoolOwned: Get<u32>;

		/// The maximum number of contract address can added to the pool
		#[pallet::constant]
		type MaxPoolTarget: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		type IWhitelist: IWhitelist<Self::AccountId>;
	}

	//** Storages **//
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Holding the all the pool data
	#[pallet::storage]
	pub type Pools<T: Config> = StorageMap<_, Twox64Concat, ID, FundingPool<T::AccountId>>;

	/// Holding the pool owned
	#[pallet::storage]
	#[pallet::getter(fn pool_owned)]
	pub type PoolOwned<T: Config> =
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
		/// Can not convert pool id to account
		IntoAccountFail,
		IntoU32Fail,
		/// Origin not the owner
		NotTheOwner,
		PoolNotExist,
		ExceedMaxPoolOwned,
		ExceedPoolTarget,
		NotReachMinPoolBalance,
		LessThanMinTxLimit,
		GreaterThanMaxTxLimit,
		LessThanMinDiscountPercent,
		GreaterThanMinDiscountPercent,
		WhitelistedPool,
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
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_pool(50u64))]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			targets: Vec<H160>,
			value: BalanceOf<T>,
			discount: Permill,
			tx_limit: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let pool_config = Self::new_pool()?;
			ensure!(
				Pools::<T>::get(pool_config.id).is_none(),
				<Error<T>>::PoolIdExisted
			);
			ensure!(
				T::Currency::free_balance(&sender) > value,
				pallet_balances::Error::<T>::InsufficientBalance
			);
			ensure!(
				balance_try_to_u128::<<T as pallet::Config>::Currency, T::AccountId>(value)? >=
					T::MinPoolBalance::get(),
				Error::<T>::NotReachMinPoolBalance
			);
			ensure!(
				tx_limit >= T::MinTxLimit::get(),
				Error::<T>::LessThanMinTxLimit
			);
			ensure!(
				tx_limit <= T::MaxTxLimit::get(),
				Error::<T>::GreaterThanMaxTxLimit
			);
			ensure!(
				discount >= T::MinDiscountPercent::get(),
				Error::<T>::LessThanMinDiscountPercent
			);
			ensure!(
				discount <= T::MaxDiscountPercent::get(),
				Error::<T>::GreaterThanMinDiscountPercent
			);
			ensure! {
				Self::usize_try_to_u32(targets.len())? <= T::MaxPoolTarget::get(),
				<Error<T>>::ExceedPoolTarget
			}

			let new_pool = FundingPool {
				id: pool_config.id,
				owner: sender.clone(),
				value: balance_try_to_u128::<<T as pallet::Config>::Currency, T::AccountId>(value)?,
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
					if target_vec.try_push(target).is_err() {
						return Err(())
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
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_pool(50u64))]
		#[pallet::call_index(1)]
		#[transactional]
		pub fn withdraw_pool(origin: OriginFor<T>, pool_id: ID) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// SBP Review - Recommended to use idiomatic rust functions `is_some()`
			// <Pools<T>>::get(pool_id).is_some()
			ensure!(Pools::<T>::get(pool_id) != None, <Error<T>>::PoolNotExist);
			ensure!(
				Self::is_pool_owner(&pool_id, &sender)?,
				<Error<T>>::NotTheOwner
			);

			if let Some(pool) = into_account::<T::AccountId>(pool_id) {
				transfer_all::<T, <T as pallet::Config>::Currency>(&pool, &sender, false)?;
				PoolOwned::<T>::try_mutate(&sender, |pool_owned| {
					if let Some(ind) = pool_owned.iter().position(|&id| id == pool_id) {
						pool_owned.swap_remove(ind);
						return Ok(())
					}
					Err(())
				})
				.map_err(|_| <Error<T>>::PoolNotExist)?;
				Pools::<T>::remove(pool_id);
				Targets::<T>::remove(pool_id);
				Self::deposit_event(Event::Withdrew { id: pool_id });
				Ok(())
			} else {
				Err(Error::<T>::IntoAccountFail.into())
			}
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
		#[pallet::weight(<T as pallet::Config>::WeightInfo::new_targets(50u64))]
		#[pallet::call_index(2)]
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
					if target_vec.try_push(target).is_err() {
						return Err(())
					}
				}
				Ok(())
			})
			.map_err(|_| <Error<T>>::ExceedPoolTarget)?;

			Ok(())
		}

		/// Set a pool's name. The name should be a UTF-8-encoded string by convention, though
		/// we don't check it. Fail if the pool is not exist or the origin is not the owner of the
		/// pool.
		///
		/// The name may not be more than `T::MaxLength` bytes, nor less than `T::MinLength` bytes
		/// which defined in the name pallet's config.
		///
		/// If the pool doesn't already have a name, then a fee of `ReservationFee` is reserved
		/// in the account.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// # <weight>
		/// - O(1).
		/// - At most one balance operation.
		/// - One storage read/write.
		/// - One event.
		/// # </weight>
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_pool_name(50u64))]
		pub fn set_pool_name(origin: OriginFor<T>, pool_id: ID, name: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(Pools::<T>::get(pool_id) != None, <Error<T>>::PoolNotExist);
			ensure!(
				Self::is_pool_owner(&pool_id, &sender)?,
				<Error<T>>::NotTheOwner
			);

			T::PoolName::set_name(sender, pool_id, name)?;

			Ok(())
		}

		/// Clear a pool's name and return the deposit. Fails if the pool was not named, not exist
		/// or the origin is not the owner of the pool.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// # <weight>
		/// - O(1).
		/// - One balance operation.
		/// - One storage read/write.
		/// - One event.
		/// # </weight>
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::clear_pool_name(50u32))]
		pub fn clear_pool_name(origin: OriginFor<T>, pool_id: ID) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(Pools::<T>::get(pool_id) != None, <Error<T>>::PoolNotExist);
			ensure!(
				Self::is_pool_owner(&pool_id, &sender)?,
				<Error<T>>::NotTheOwner
			);

			T::PoolName::clear_name(sender, pool_id)?;

			Ok(())
		}

		/// Remove a name and take charge of the deposit.
		///
		/// Fails if `target` has not been named. The deposit is dealt with through `T::Slashed`
		/// imbalance handler.
		///
		/// the dispatch origin for this call must be _Root_.
		///
		/// # <weight>
		/// - O(1).
		/// - One unbalanced handler (probably a balance transfer)
		/// - One storage read/write.
		/// - One event.
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::kill_pool_name(50u32))]
		#[pallet::call_index(5)]
		pub fn kill_pool_name(origin: OriginFor<T>, pool_id: ID) -> DispatchResult {
			ensure_root(origin)?;

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

		fn usize_try_to_u32(input: usize) -> Result<u32, Error<T>> {
			match input.try_into().ok() {
				Some(val) => Ok(val),
				None => Err(<Error<T>>::IntoU32Fail),
			}
		}

		fn is_pool_owner(pool_id: &ID, owner: &T::AccountId) -> Result<bool, Error<T>> {
			match Pools::<T>::get(pool_id) {
				Some(pool) => Ok(pool.owner == *owner),
				None => Err(<Error<T>>::PoolNotExist),
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {
			assert!(T::MinDiscountPercent::get() <= T::MaxDiscountPercent::get());
		}
	}

	impl<T: Config> CustomPool<T::AccountId> for Pallet<T> {
		fn join(sender: T::AccountId, pool_id: ID) -> DispatchResult {
			ensure!(Self::is_pool(pool_id), Error::<T>::PoolNotExist);

			if T::IWhitelist::is_whitelist(pool_id) {
				T::IWhitelist::insert_whitelist(pool_id, sender)?;
				return Err(<Error<T>>::WhitelistedPool.into())
			}
			Ok(())
		}
		fn leave(_sender: T::AccountId) -> DispatchResult {
			Ok(())
		}

		fn is_pool(pool_id: ID) -> bool {
			match Pools::<T>::get(pool_id) {
				Some(_) => true,
				None => false,
			}
		}

		fn get_service(pool_id: ID) -> Option<CustomService<T::AccountId>> {
			if let Some(pool) = Pools::<T>::get(pool_id) {
				let targets = Targets::<T>::get(pool_id);
				return Some(CustomService::new(
					targets.to_vec(),
					pool.tx_limit,
					pool.discount,
					pool.owner,
				))
			}
			None
		}

		fn get_pool_owner(pool_id: ID) -> Option<T::AccountId> {
			if let Some(pool) = Pools::<T>::get(pool_id) {
				return Some(pool.owner)
			}
			return None
		}

		/// Add new funding-pool with default values, return pool_id
		///
		/// ** Should be used for benchmarking only!!! **
		#[cfg(feature = "runtime-benchmarks")]
		fn add_default(owner: T::AccountId, pool_id: ID) {
			let funding_pool = FundingPool {
				id: pool_id,
				owner: owner.clone(),
				value: 0_u128,
				discount: Permill::from_percent(0),
				tx_limit: 0_u32,
			};

			Pools::<T>::insert(pool_id, funding_pool);
			let _ = PoolOwned::<T>::try_mutate(&owner, |pool_vec| pool_vec.try_push(pool_id))
				.map_err(|_| <Error<T>>::ExceedMaxPoolOwned);
		}
	}
}
