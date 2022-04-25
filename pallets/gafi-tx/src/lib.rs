
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
	traits::{Currency, Imbalance, OnUnbalanced},
};
use frame_system::pallet_prelude::*;
use gafi_primitives::{
	pool::{PlayerTicket, TicketType}
};
pub use pallet::*;
use pallet_evm::{AddressMapping, GasWeightMapping};
use pallet_evm::OnChargeEVMTransaction;
use sp_core::{H160, U256};
use pallet_evm::FeeCalculator;

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
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	pub type NegativeImbalanceOf<C, T> =
		<C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config + pallet_balances::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		
		/// The currency mechanism.
		type Currency: Currency<Self::AccountId>;
		
		/// Customize OnChargeEVMTransaction
		type OnChargeEVMTxHandler: OnChargeEVMTransaction<Self>;

		/// Substrate - EVM Address Mapping
		type AddressMapping: AddressMapping<Self::AccountId>;

		/// To use tickets
		type PlayerTicket: PlayerTicket<Self::AccountId>;
	}

	//** STORAGE **//

	/// Holding gas price value
	#[pallet::storage]
	pub type GasPrice<T: Config> = StorageValue<_, U256, ValueQuery>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub gas_price: U256,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				gas_price: U256::from(100_000_000_000u128),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			GasPrice::<T>::put(self.gas_price);
		}
	}

	// Errors.
	#[derive(PartialEq)]
	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SetGasPrice {value: U256},
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Set Gas Price
		///
		/// The root must be Signed
		///
		/// Parameters:
		/// - `new_gas_price`: new gas_price value
		///
		/// Weight: `O(1)`
		#[pallet::weight(0)]
		pub fn set_gas_price(origin: OriginFor<T>, new_gas_price: U256) -> DispatchResult {
			ensure_root(origin)?;
			GasPrice::<T>::put(new_gas_price);
			Self::deposit_event(Event::<T>::SetGasPrice{value: new_gas_price});
			Ok(())
		}

	}

	impl<T: Config> FeeCalculator for Pallet<T> {
		fn min_gas_price() -> sp_core::U256 { 
			GasPrice::<T>::get()
		 }
	}
}

pub struct GafiEVMCurrencyAdapter<C, OU>(sp_std::marker::PhantomData<(C, OU)>);

impl<T, C, OU> OnChargeEVMTransaction<T> for GafiEVMCurrencyAdapter<C, OU>
where
	T: Config,
	C: Currency<<T as frame_system::Config>::AccountId>,
	C::PositiveImbalance: Imbalance<
		<C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
		Opposite = C::NegativeImbalance,
	>,
	C::NegativeImbalance: Imbalance<
		<C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
		Opposite = C::PositiveImbalance,
	>,
	OU: OnUnbalanced<NegativeImbalanceOf<C, T>>,
{
	type LiquidityInfo =
		<<T as pallet::Config>::OnChargeEVMTxHandler as OnChargeEVMTransaction<T>>::LiquidityInfo;
	fn withdraw_fee(who: &H160, fee: U256) -> Result<Self::LiquidityInfo, pallet_evm::Error<T>> {
		T::OnChargeEVMTxHandler::withdraw_fee(who, fee)
	}

	/// Steps
	/// 1. Get player ticket to reduce the transaction fee
	/// 2. Use ticket 
	fn correct_and_deposit_fee(
		who: &H160,
		corrected_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	) {
		let mut service_fee = corrected_fee;
		let account_id: T::AccountId = <T as pallet::Config>::AddressMapping::into_account_id(*who);
		if let Some(ticket) = T::PlayerTicket::use_ticket(account_id) {
			if let Some(service) = T::PlayerTicket::get_service(ticket) {
				match ticket {
					TicketType::Staking(_) | TicketType::Upfront(_) => {
						let discount_fee = service_fee.saturating_mul(U256::from(service.discount)).checked_div(U256::from(100u64));
						service_fee = service_fee.saturating_sub(discount_fee.unwrap_or_else(|| U256::from(0u64)));
					},
					TicketType::Sponsored(_) => {
						let discount_fee = service_fee.saturating_mul(U256::from(service.discount)).checked_div(U256::from(100u64));
						let sponsored_fee = service_fee.saturating_sub(discount_fee.unwrap_or_else(|| U256::from(0u64)));
						let sponsor_fee = service_fee - sponsored_fee;
						{
							// let sponsor = 
						}
						service_fee = sponsored_fee;
					}
				}
			}
		}
		T::OnChargeEVMTxHandler::correct_and_deposit_fee(who, service_fee, already_withdrawn)
	}

	fn pay_priority_fee(tip: U256) {
		T::OnChargeEVMTxHandler::pay_priority_fee(tip)
	}
}

pub struct GafiGasWeightMapping;

impl GasWeightMapping for GafiGasWeightMapping {
	fn gas_to_weight(gas: u64) -> Weight {
		gas as Weight
	}

	fn weight_to_gas(weight: Weight) -> u64 {
		weight as u64
	}
}
