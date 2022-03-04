#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::Weight;
pub use pallet::*;
use pallet_evm::GasWeightMapping;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// struct AurGasWeightMapping;

// impl GasWeightMapping for AurGasWeightMapping {
// 	fn gas_to_weight(gas: u64) -> Weight {
// 		gas as Weight
// 	}

// 	fn weight_to_gas(weight: Weight) -> u64 {
// 		weight as u64
// 	}
// }

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{print, traits::Hash},
		traits::{
			tokens::{ExistenceRequirement, WithdrawReasons},
			Currency, Randomness,
		},
		transactional, weights,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::{H160, H256, U256};
	use sp_io::hashing::blake2_256;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		type GameRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// Errors.
	#[derive(PartialEq)]
	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	// Storage

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn call(
			origin: OriginFor<T>,
			source: H160,
			target: H160,
			input: Vec<u8>,
			value: U256,
			gas_limit: u64,
			max_fee_per_gas: U256,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Vec<(H160, Vec<H256>)>,
		) -> DispatchResult {
			// pallet_evm::Call::call{source, target, input, value, gas_limit, max_fee_per_gas, max_priority_fee_per_gas, nonce, access_list};
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {}
}
