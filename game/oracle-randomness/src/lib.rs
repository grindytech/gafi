#![cfg_attr(not(feature = "std"), no_std)]

use gafi_support::game::GameRandomness;
pub use pallet::*;
use sp_runtime::traits::TrailingZeroInput;

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
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Payload used to hold seed data required to submit a transaction.
	#[derive(
		Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo, MaxEncodedLen,
	)]
	pub struct SeedPayload<BlockNumber, Seed> {
		block_number: BlockNumber,
		seed: Seed,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		type RandomAttemps: Get<u32>;

		#[pallet::constant]
		type SeedLength: Get<u32>;
	}

	/// Storing random seed generated.
	#[pallet::storage]
	pub(crate) type RandomSeed<T: Config> =
		StorageValue<_, SeedPayload<BlockNumberFor<T>, BoundedVec<u8, T::SeedLength>>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		InvalidSeed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T: Config> Pallet<T> {
		pub(crate) fn gen_random(seed: &[u8]) -> Result<u32, Error<T>> {
			match <u32>::decode(&mut TrailingZeroInput::new(seed.as_ref())) {
				Ok(random) => Ok(random),
				Err(_) => Err(Error::<T>::InvalidSeed),
			}
		}

		pub fn random_bias(seed: &[u8], total: u32, attempts: u32) -> Option<u32> {
			let mut random_number = Self::gen_random(&seed);

			for _ in 1..attempts {
				if let Ok(rand_val) = random_number {
					if rand_val < u32::MAX.saturating_sub(u32::MAX.wrapping_rem(total)) {
						break
					}
					random_number = Self::gen_random(&seed);
				}
			}
			if let Ok(rand_val) = random_number {
				return Some((rand_val.wrapping_rem(total)).saturating_add(1))
			}

			None
		}
	}

	impl<T: Config> GameRandomness for Pallet<T> {
		/// Generates a random number between 1 and `total` (inclusive).
		/// This function repeats the process up to `RandomAttemps` times if
		/// the number falls within the overflow range of the modulo operation to mitigate modulo
		/// bias.
		///
		/// Returns `None` if `total` is 0.
		fn random_number(total: u32) -> Option<u32> {
			if total == 0 {
				return None
			}

			let seed_payload = RandomSeed::<T>::get();

			if let Some(payload) = seed_payload {
				return Self::random_bias(&payload.seed, total, T::RandomAttemps::get())
			}
			None
		}
	}
}
