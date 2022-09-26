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
};
use frame_system::pallet_prelude::*;
use gafi_primitives::{
	constant::ID,
	custom_services::CustomPool,
	whitelist::{IWhitelist, WhitelistPool},
};

pub use pallet::*;
use scale_info::prelude::{format, string::String};
use sp_std::{prelude::*, str};

use frame_system::offchain::{CreateSignedTransaction, SubmitTransaction};
use rustc_hex::ToHex;
use sp_core::crypto::KeyTypeId;
use sp_runtime::offchain::{http, Duration};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"gafi");
pub const UNSIGNED_TXS_PRIORITY: u64 = 10;

pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);
	pub struct TestAuthId;

	// implemented for runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
		
		/// Traits for joining the pool
		type WhitelistPool: WhitelistPool<Self::AccountId>;

		/// Traits CustomPool
		type SponsoredPool: CustomPool<Self::AccountId>;

		/// Maximum length of whitelist query api
		type MaxWhitelistLength: Get<u32>;

		/// The reserve fee when enable pool whitelist
		#[pallet::constant]
		type WhitelistFee: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// The request to apply to the pool's whitelist
	#[pallet::storage]
	pub type Whitelist<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, ID>;

	/// Get whitelist url
	#[pallet::storage]
	pub type WhitelistSource<T: Config> =
		StorageMap<_, Twox64Concat, ID, (BoundedVec<u8, T::MaxWhitelistLength>, BalanceOf<T>)>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Whitelisted { sender: T::AccountId, pool_id: ID },
		WhitelistEnabled { pool_id: ID, url: Vec<u8> },
		WhitelistChanged { pool_id: ID, url: Vec<u8> },
		WhitelistWithdrew { pool_id: ID },
	}

	#[pallet::error]
	pub enum Error<T> {
		NotWhitelist,
		AlreadyWhitelist,
		NotPoolOwner,
		PoolNotFound,
		PoolNotWhitelist,
		URLTooLong,
		AlreadyJoined,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			let res = Self::verify_whitelist_and_send_raw_unsign(block_number);
			if let Err(e) = res {
				log::error!("Error: {}", e);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Approve whitelist
		/// 
		/// The pool owner approves the request whitelist of player
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `player`: the player, whose request to whitelist
		/// - `pool_id`: pool id
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::approve_whitelist(50u32))]
		pub fn approve_whitelist(
			origin: OriginFor<T>,
			player: T::AccountId,
			pool_id: ID,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::is_pool_owner(pool_id, &sender)?;

			ensure!(
				Self::is_whitelist_player(&player, pool_id),
				<Error<T>>::NotWhitelist
			);

			T::WhitelistPool::join_pool(&player, pool_id)?;
			Whitelist::<T>::remove(player.clone());
			Self::deposit_event(Event::<T>::Whitelisted {
				sender: player,
				pool_id,
			});
			Ok(())
		}

		/// Approve whitelist unsigned
		/// 
		/// Unsigned approve the request whitelist of players, this function disable by default
		/// This function should be called only by offchain-worker
		///
		/// Unsign
		///
		/// Parameters:
		/// - `player`: the player, whose request to whitelist
		/// - `pool_id`: pool id
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::approve_whitelist_unsigned(50u32))]
		pub fn approve_whitelist_unsigned(
			origin: OriginFor<T>,
			player: T::AccountId,
			pool_id: ID,
		) -> DispatchResult {
			ensure_none(origin)?;

			ensure!(
				Self::is_whitelist_player(&player, pool_id),
				<Error<T>>::NotWhitelist
			);

			T::WhitelistPool::join_pool(&player, pool_id)?;
			Whitelist::<T>::remove(player.clone());

			Self::deposit_event(Event::<T>::Whitelisted {
				sender: player,
				pool_id,
			});
			Ok(())
		}

		/// Apply whitelist
		/// 
		/// Player request to whitelist
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `pool_id`: pool id
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::apply_whitelist(50u32))]
		pub fn apply_whitelist(origin: OriginFor<T>, pool_id: ID) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::insert_whitelist(pool_id, sender)?;

			Ok(())
		}

		/// Enable whitelist
		/// 
		/// Pool owners enable whitelist access function
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `pool_id`: pool id
		/// - `url`: verify api
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::enable_whitelist(50u32))]
		pub fn enable_whitelist(origin: OriginFor<T>, pool_id: ID, url: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::is_pool_owner(pool_id, &sender)?;

			let bounded_url: BoundedVec<_, _> =
				url.clone().try_into().map_err(|()| Error::<T>::URLTooLong)?;

			let deposit = T::WhitelistFee::get();
			if <WhitelistSource<T>>::get(pool_id) == None {
				T::Currency::reserve(&sender, deposit)?;
				Self::deposit_event(Event::<T>::WhitelistEnabled { pool_id, url });
			} else {
				Self::deposit_event(Event::<T>::WhitelistChanged { pool_id, url });
			}
			<WhitelistSource<T>>::insert(pool_id, (bounded_url, deposit));
			Ok(())
		}

		/// Withdraw whitelist
		/// 
		/// Pool owners withdraw whitelist
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `pool_id`: pool id
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_whitelist(50u32))]
		pub fn withdraw_whitelist(origin: OriginFor<T>, pool_id: ID) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::is_pool_owner(pool_id, &sender)?;

			if let Some(source) = WhitelistSource::<T>::get(pool_id) {
				let deposit = source.1;
				T::Currency::unreserve(&sender, deposit);
			} else {
				return Err(<Error::<T>>::PoolNotWhitelist.into());
			}

			<WhitelistSource<T>>::remove(pool_id);

			Self::deposit_event(Event::<T>::WhitelistWithdrew { pool_id });

			Ok(())
		}
	}

	impl<T: Config> IWhitelist<T::AccountId> for Pallet<T> {
		fn is_whitelist(pool_id: ID) -> bool {
			match WhitelistSource::<T>::get(pool_id) {
				Some(_) => true,
				None => false,
			}
		}

		fn insert_whitelist(pool_id: ID, player: T::AccountId) -> Result<(), &'static str> {
			ensure!(T::SponsoredPool::is_pool(pool_id), Error::<T>::PoolNotFound);

			ensure!(Self::is_whitelist(pool_id), <Error::<T>>::PoolNotWhitelist,);

			ensure!(
				!Self::is_whitelist_player(&player, pool_id),
				<Error::<T>>::AlreadyWhitelist,
			);

			ensure!(
				!T::WhitelistPool::is_joined_pool(&player, pool_id),
				<Error::<T>>::AlreadyJoined,
			);

			Whitelist::<T>::insert(player, pool_id);
			Ok(())
		}
	}

	// whitelist implement
	impl<T: Config> Pallet<T> {
		pub fn verify_whitelist_and_send_raw_unsign(
			_block_number: T::BlockNumber,
		) -> Result<(), &'static str> {
			for query in Whitelist::<T>::iter() {
				let player = query.0;
				let pool_id = query.1;

				if let Some(url) = Self::get_url(pool_id) {
					let api = Self::get_api(&url, pool_id, &player);
					let _ = Self::verify_and_approve(&api, player, pool_id);
				}
			}
			return Ok(())
		}

		pub fn verify_and_approve(
			uri: &str,
			player: T::AccountId,
			pool_id: ID,
		) -> Result<(), &'static str> {
			let verify = Self::fetch_whitelist(&uri);

			if verify == Ok(true) {
				let call = Call::approve_whitelist_unsigned { player, pool_id };

				let _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
					.map_err(|_| {
						log::error!("Failed in offchain_unsigned_tx");
					});
			}

			Ok(())
		}

		fn is_whitelist_player(player: &T::AccountId, pool_id: ID) -> bool {
			if let Some(id) = Whitelist::<T>::get(player) {
				if id == pool_id {
					return true
				}
			}
			false
		}

		pub fn fetch_whitelist(url: &str) -> Result<bool, http::Error> {
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));

			let request = http::Request::get(url);

			let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;

			let response =
				pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;

			if response.code != 200 {
				log::warn!("Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown)
			}

			let body = response.body().collect::<Vec<u8>>();

			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::warn!("No UTF8 body");
				http::Error::Unknown
			})?;

			let verify: bool = match body_str {
				"true" => true,
				_ => false,
			};

			Ok(verify)
		}

		pub fn get_api(link: &str, pool_id: ID, player: &T::AccountId) -> String {
			let pool_id_hex: String = pool_id.to_hex();

			let address = player.encode();

			let hex_address: String = address.to_hex();
			let uri = format!("{link}?pool_id={pool_id_hex}&address={hex_address}");
			uri
		}

		pub fn get_url(pool_id: ID) -> Option<String> {
			if let Some(source) = WhitelistSource::<T>::get(pool_id) {
				if let Ok(url) = sp_std::str::from_utf8(&source.0) {
					return Some(format!("{}", url))
				}
			}
			return None
		}

		fn is_pool_owner(pool_id: ID, sender: &T::AccountId) -> Result<(), Error<T>> {
			if let Some(owner) = T::SponsoredPool::get_pool_owner(pool_id) {
				if owner == *sender {
					return Ok(())
				} else {
					return Err(Error::<T>::NotPoolOwner)
				}
			}
			return Err(Error::<T>::PoolNotFound)
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			let valid_tx = |provide| {
				ValidTransaction::with_tag_prefix("pallet-pool")
					.priority(UNSIGNED_TXS_PRIORITY) // please define `UNSIGNED_TXS_PRIORITY` before this line
					.and_provides([&provide])
					.longevity(3)
					.propagate(true)
					.build()
			};

			match call {
				Call::approve_whitelist_unsigned { pool_id, player } => match source {
					TransactionSource::Local | TransactionSource::InBlock =>
						valid_tx(b"approve_whitelist_unsigned".to_vec()),
					_ => InvalidTransaction::Call.into(),
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}
}
