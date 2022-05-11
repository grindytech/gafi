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
use frame_support::traits::tokens::{ExistenceRequirement, WithdrawReasons};
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, Imbalance, OnUnbalanced},
};
use frame_system::pallet_prelude::*;
use gafi_primitives::{
	constant::ID,
	game_creator::GetGameCreator,
	pool::{PlayerTicket, TicketType},
};
pub use pallet::*;
use pallet_evm::FeeCalculator;
use pallet_evm::OnChargeEVMTransaction;
use pallet_evm::{AddressMapping, GasWeightMapping};
use sp_core::{H160, U256};
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

		/// percentage of transaction fee reward to game-creator
		#[pallet::constant]
		type GameCreatorReward: Get<u8>;

		type GetGameCreator: GetGameCreator<Self::AccountId>;
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
	pub enum Error<T> {
		IntoBalanceFail,
		IntoAccountFail,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SetGasPrice { value: U256 },
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
			Self::deposit_event(Event::<T>::SetGasPrice {
				value: new_gas_price,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn u128_try_to_balance(input: u128) -> Result<BalanceOf<T>, Error<T>> {
			match input.try_into().ok() {
				Some(val) => Ok(val),
				None => Err(<Error<T>>::IntoBalanceFail),
			}
		}

		pub fn u128_to_balance(input: u128) -> BalanceOf<T> {
			input.try_into().ok().unwrap_or_default()
		}

		pub fn into_account(id: ID) -> Result<T::AccountId, Error<T>> {
			match T::AccountId::decode(&mut &id[..]) {
				Ok(account) => Ok(account),
				Err(_) => Err(<Error<T>>::IntoAccountFail),
			}
		}

		pub fn correct_and_deposit_fee_sponsored(
			pool_id: ID,
			targets: Vec<H160>,
			target: Option<H160>,
			service_fee: U256,
			discount: u8,
		) -> Option<U256> {
			if !Self::is_target(targets, target) {
				return None;
			}

			if let Ok(sponsor) = Pallet::<T>::into_account(pool_id) {
				let sponsor_fee = service_fee
					.saturating_mul(U256::from(discount))
					.checked_div(U256::from(100u64))
					.unwrap_or_else(|| U256::from(0u64));

				let player_fee = service_fee.saturating_sub(sponsor_fee);

				if let Ok(fee) = Pallet::<T>::u128_try_to_balance(sponsor_fee.as_u128()) {
					if let Ok(_) = <T as pallet::Config>::Currency::withdraw(
						&sponsor,
						fee,
						WithdrawReasons::FEE,
						ExistenceRequirement::KeepAlive,
					) {
						return Some(player_fee);
					}
				}
			}
			None
		}

		fn is_target(targets: Vec<H160>, target: Option<H160>) -> bool {
			if let Some(tar) = target {
				return targets.contains(&tar);
			}
			false
		}

		pub fn correct_and_deposit_fee_service(service_fee: U256, discount: u8) -> Option<U256> {
			let discount_fee = service_fee
				.saturating_mul(U256::from(discount))
				.checked_div(U256::from(100u64));

			Some(service_fee.saturating_sub(discount_fee.unwrap_or_else(|| U256::from(0u64))))
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
		target: Option<H160>,
		corrected_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	) {
		let mut service_fee = corrected_fee;
		// get mapping account id
		let account_id: T::AccountId = <T as pallet::Config>::AddressMapping::into_account_id(*who);
		// get transaction service based on player's service
		if let Some(ticket) = T::PlayerTicket::use_ticket(account_id) {
			if let Some(service) = T::PlayerTicket::get_service(ticket) {
				match ticket {
					TicketType::Staking(_) | TicketType::Upfront(_) => {
						if let Some(fee) = Pallet::<T>::correct_and_deposit_fee_service(
							service_fee,
							service.discount,
						) {
							service_fee = fee;
						}
					}
					TicketType::Sponsored(pool_id) => {
						let targets = T::PlayerTicket::get_targets(pool_id);
						if let Some(fee) = Pallet::<T>::correct_and_deposit_fee_sponsored(
							pool_id,
							targets,
							target,
							service_fee,
							service.discount,
						) {
							service_fee = fee;
						}
					}
				}
			}
		}

		if let Some(contract) = target {
			if let Some(creator) = T::GetGameCreator::get_game_creator(&contract) {
				let reward = service_fee
					.saturating_mul(U256::from(T::GameCreatorReward::get()))
					.checked_div(U256::from(100u64))
					.unwrap_or_else(|| U256::from(0u64));
				<T as Config>::Currency::deposit_into_existing(
					&creator,
					Pallet::<T>::u128_to_balance(reward.as_u128()),
				);
			}
		}

		T::OnChargeEVMTxHandler::correct_and_deposit_fee(
			who,
			target,
			service_fee,
			already_withdrawn,
		)
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
