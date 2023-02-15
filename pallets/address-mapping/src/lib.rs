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
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{Currency, Get, ReservableCurrency},
	transactional, Twox64Concat,
};
use frame_system::pallet_prelude::*;
use gu_convertor::into_account;
use gu_currency::transfer_all;
use gu_ethereum::{eth_recover, to_ascii_hex, EcdsaSignature, EthereumAddress};
pub use pallet::*;
use pallet_evm::AddressMapping;
use sp_core::{crypto::AccountId32, H160};
use sp_io::hashing::blake2_256;

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
	use crate::weights::WeightInfo;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	pub type NegativeImbalanceOf<C, T> =
		<C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config {
		/// The overarching event type.
				type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Message Prefix for signing messages using ecdsa signature
		#[pallet::constant]
		type MessagePrefix: Get<&'static [u8]>;

		#[pallet::constant]
		type ReservationFee: Get<BalanceOf<Self>>;
	}

	// holding AccountId32 address that bonded for H160 address
	#[pallet::storage]
	pub type H160Mapping<T: Config> = StorageMap<_, Twox64Concat, H160, AccountId32>;

	// holding H160 address that bonded for AccountId32 address
	#[pallet::storage]
	pub type Id32Mapping<T: Config> =
		StorageMap<_, Twox64Concat, AccountId32, (H160, BalanceOf<T>)>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Bonded { sender: T::AccountId, address: H160 },
		Unbonded { sender: T::AccountId, address: H160 },
	}

	#[derive(PartialEq)]
	#[pallet::error]
	pub enum Error<T> {
		// when can't verify the signature and message
		SignatureOrAddressNotCorrect,
		// Substrate address or EVM address already bond
		AlreadyBond,
		// Making unbond with non-bonding account
		NonbondAccount,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		[u8; 32]: From<<T as frame_system::Config>::AccountId>,
		AccountId32: From<<T as frame_system::Config>::AccountId>,
	{
		/// Bond Substrate(H256) address with EVM(H160) address
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `signature`: signature of the address that signed the message contain hex format of
		///   origin
		///
		/// - `address`: EVM(H160) address that you want to bond
		///
		/// - `withdraw`: true/false withdraw all the balance of original account of address trasfer
		///   to
		/// the origin, always KeepAlive original address
		///
		/// Emits `Bonded` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::bond(100u32))]
		#[transactional]
		pub fn bond(
			origin: OriginFor<T>,
			signature: [u8; 65],
			address: H160,
			withdraw: bool,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let account_id: AccountId32 = sender.clone().into();

			ensure!(
				Id32Mapping::<T>::get(account_id.clone()).is_none() &&
					H160Mapping::<T>::get(address).is_none(),
				<Error<T>>::AlreadyBond
			);
			ensure!(
				Self::verify_bond(sender.clone(), signature, address.to_fixed_bytes()),
				<Error<T>>::SignatureOrAddressNotCorrect,
			);

			<T as pallet::Config>::Currency::reserve(&sender, T::ReservationFee::get())?;

			if withdraw {
				let id = Self::into_account_id(address);
				if let Some(from) = into_account::<T::AccountId>(id.into()) {
					transfer_all::<T, <T as pallet::Config>::Currency>(&from, &sender, true)?;
				}
			}

			Self::insert_pair_bond(address, account_id);
			Self::deposit_event(Event::Bonded { sender, address });
			Ok(())
		}

		/// Unbonded Substrate(H256) address to EVM(H160) address remove
		/// the bond so both two accounts will be using the default AddressMapping
		///
		/// The origin must be Signed
		///
		/// Emits `Unbonded` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::weight(<T as pallet::Config>::WeightInfo::unbond(100u32))]
		#[transactional]
		pub fn unbond(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let account_id: AccountId32 = sender.clone().into();

			let mapping_data = <Id32Mapping<T>>::get(account_id);
			ensure!(mapping_data.is_some(), <Error<T>>::NonbondAccount);

			let (evm_address, deposit) = mapping_data.unwrap();
			let id32_address = <H160Mapping<T>>::get(evm_address);
			ensure!(id32_address.is_some(), <Error<T>>::NonbondAccount);

			<T as pallet::Config>::Currency::unreserve(&sender, deposit);

			Self::remove_pair_bond(evm_address, id32_address.unwrap());
			Self::deposit_event(Event::Unbonded {
				sender,
				address: evm_address,
			});
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T>
where
	[u8; 32]: From<<T as frame_system::Config>::AccountId>,
{
	pub fn verify_bond(sender: T::AccountId, sig: [u8; 65], address: [u8; 20]) -> bool {
		let sig_converter = EcdsaSignature(sig);
		let address_convert = EthereumAddress(address);
		let who = sender.using_encoded(to_ascii_hex);
		let signer = eth_recover(&sig_converter, &who, &[][..], T::MessagePrefix::get());
		signer == Some(address_convert)
	}

	pub fn get_evm_address(account_id: AccountId32) -> Option<H160> {
		let data: [u8; 32] = account_id.into();
		if data.starts_with(b"evm:") {
			Some(H160::from_slice(&data[4..24]))
		} else {
			None
		}
	}

	pub fn get_default_evm_address(account_id: AccountId32) -> H160 {
		let payload = (b"evm:", account_id);
		H160::from_slice(&payload.using_encoded(blake2_256)[0..20])
	}

	pub fn get_or_create_evm_address(account_id: AccountId32) -> H160 {
		Self::get_evm_address(account_id.clone())
			.unwrap_or_else(|| Self::get_default_evm_address(account_id))
	}

	fn insert_pair_bond(address: H160, account_id: AccountId32)
	where
		sp_runtime::AccountId32: From<<T as frame_system::Config>::AccountId>,
	{
		let origin_account_id: AccountId32 = OriginAddressMapping::into_account_id(address);
		let origin_address: H160 = Self::get_or_create_evm_address(account_id.clone());

		<H160Mapping<T>>::insert(address, account_id.clone());
		<Id32Mapping<T>>::insert(account_id, (address, T::ReservationFee::get()));

		<H160Mapping<T>>::insert(origin_address, origin_account_id.clone());
		<Id32Mapping<T>>::insert(
			origin_account_id,
			(origin_address, T::ReservationFee::get()),
		);
	}

	fn remove_pair_bond(address: H160, account_id: AccountId32)
	where
		sp_runtime::AccountId32: From<<T as frame_system::Config>::AccountId>,
	{
		<H160Mapping<T>>::remove(address);
		<Id32Mapping<T>>::remove(account_id.clone());

		let origin_address: H160 = Self::get_or_create_evm_address(account_id);
		let origin_account_id = H160Mapping::<T>::get(origin_address)
			.unwrap_or_else(|| OriginAddressMapping::into_account_id(address));

		<H160Mapping<T>>::remove(origin_address);
		<Id32Mapping<T>>::remove(origin_account_id);
	}
}

struct OriginAddressMapping;

impl pallet_evm::AddressMapping<AccountId32> for OriginAddressMapping {
	fn into_account_id(address: H160) -> AccountId32 {
		let mut data: [u8; 32] = [0u8; 32];
		data[0..4].copy_from_slice(b"evm:");
		data[4..24].copy_from_slice(&address[..]);
		AccountId32::from(data)
	}
}

impl<T> pallet_evm::AddressMapping<AccountId32> for Pallet<T>
where
	T: Config,
{
	fn into_account_id(address: H160) -> AccountId32 {
		if let Some(account_id) = H160Mapping::<T>::get(address) {
			account_id
		} else {
			OriginAddressMapping::into_account_id(address)
		}
	}
}
