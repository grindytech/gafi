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
	traits::{BalanceStatus, Currency, ReservableCurrency},
	transactional,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use pallet_evm::{AddressMapping, ContractCreator};
use sp_core::H160;

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

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Substrate <-> Ethereum address mapping
		type AddressMapping: AddressMapping<Self::AccountId>;

		/// A maximum number of contracts can be owned
		#[pallet::constant]
		type MaxContractOwned: Get<u32>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The verify origin contract owner function from Frontier
		type ContractCreator: ContractCreator;

		/// Balance reserve for the claim of ownership
		#[pallet::constant]
		type ReservationFee: Get<BalanceOf<Self>>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	//** STORAGE  **//
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Holing the contract owner
	#[pallet::storage]
	pub type ContractOwner<T: Config> = StorageMap<_, Twox64Concat, H160, (T::AccountId, BalanceOf<T>)>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Claimed {
			contract: H160,
			owner: T::AccountId,
		},
		Changed {
			contract: H160,
			new_owner: T::AccountId,
		},
		Withdrew {
			contract: H160,
			owner: T::AccountId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not the contract creator either game creator
		NotContractOwner,

		/// Claim the contract does not exist
		ContractNotFound,

		/// The contract had claimed
		ContractClaimed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claim the contract as an origin contract creator
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `contract`: smart-contract address to claim
		///
		/// Emits `Claimed` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::claim_contract(100u64))]
		#[transactional]
		pub fn claim_contract(origin: OriginFor<T>, contract: H160) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(
				ContractOwner::<T>::get(contract).is_none(),
				<Error<T>>::ContractClaimed
			);
			Self::verify_owner(&sender, &contract)?;

			let deposit = T::ReservationFee::get();
			<T as pallet::Config>::Currency::reserve(&sender, deposit)?;
			ContractOwner::<T>::insert(contract, (sender.clone(), deposit));

			Self::deposit_event(Event::Claimed {
				contract,
				owner: sender,
			});
			Ok(())
		}

		/// Change the contract ownership
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `contract`: smart-contract address to change
		/// - `new_owner`: new contract owner
		///
		/// Emits `Changed` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::change_ownership(100u64))]
		#[transactional]
		pub fn change_ownership(
			origin: OriginFor<T>,
			contract: H160,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::verify_owner(&sender, &contract)?;

			let mut deposit = T::ReservationFee::get();
			if let Some(data) = ContractOwner::<T>::get(contract) {
				deposit = data.1;
			}

			<T as pallet::Config>::Currency::repatriate_reserved(
				&sender,
				&new_owner,
				deposit,
				BalanceStatus::Reserved,
			)?;

			ContractOwner::<T>::insert(contract, (new_owner.clone(), deposit));
			Self::deposit_event(Event::Changed {
				contract,
				new_owner,
			});
			Ok(())
		}

		/// Withdraw the ownership
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `contract`: smart-contract address
		///
		/// Emits `Withdrew` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_contract(100u64))]
		#[transactional]
		pub fn withdraw_contract(origin: OriginFor<T>, contract: H160) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::verify_owner(&sender, &contract)?;

			let mut deposit = T::ReservationFee::get();
			if let Some(data) = ContractOwner::<T>::get(contract) {
				deposit = data.1;
			}

			ContractOwner::<T>::remove(contract);
			<T as pallet::Config>::Currency::unreserve(&sender, deposit);
			Self::deposit_event(Event::Withdrew {
				contract,
				owner: sender
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn verify_owner(sender: &T::AccountId, contract: &H160) -> Result<(), Error<T>> {
			if let Some(owner) = ContractOwner::<T>::get(&contract) {
				if owner.0 != *sender {
					return Err(Error::<T>::NotContractOwner);
				}
			} else {
				let contract_creator = Self::get_contract_creator(contract)?;
				if contract_creator != *sender {
					return Err(Error::<T>::NotContractOwner);
				}
			}
			Ok(())
		}

		fn get_contract_creator(contract: &H160) -> Result<T::AccountId, Error<T>> {
			match T::ContractCreator::get_creator(contract) {
				Some(address) => Ok(T::AddressMapping::into_account_id(address)),
				None => Err(Error::<T>::ContractNotFound),
			}
		}
	}

	impl<T: Config> gafi_primitives::pool::game_creator::GetGameCreator<T::AccountId> for Pallet<T> {
		fn get_game_creator(contract: &H160) -> Option<T::AccountId> {
			match ContractOwner::<T>::get(contract) {
				Some(contract) => Some(contract.0),
				None => None
			}
		}
	}
}
