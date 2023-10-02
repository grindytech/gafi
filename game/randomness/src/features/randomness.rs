/// Item module provides utility functions for pallet-game
use crate::*;
use gafi_support::game::GameRandomness;

impl<T: Config> Pallet<T> {
	/// Generate a random number from a given seed.
	/// Note that there is potential bias introduced by using modulus operator.
	/// You should call this function with different seed values until the random
	/// number lies within `u32::MAX - u32::MAX % n`.
	fn generate_random_number(seed: u32) -> u32 {
		let (random_seed, _) = T::Randomness::random(&(T::PalletId::get(), seed).encode());
		let random_number = <u32>::decode(&mut random_seed.as_ref())
			.expect("secure hashes should always be bigger than u32; qed");
		random_number
	}

	/// Generate a random number from the off-chain worker's random seed
	pub(crate) fn gen_random() -> Result<u32, Error<T>> {
		if let Some(seed_data) = RandomSeed::<T>::get() {
			match <u32>::decode(&mut TrailingZeroInput::new(seed_data.seed.as_ref())) {
				Ok(random) => Ok(random),
				Err(_) => Err(Error::<T>::InvalidSeed),
			}
		} else {
			Err(Error::<T>::InvalidSeed)
		}
	}
}

impl<T: Config> GameRandomness for Pallet<T> {
	/// Generates a random number between 1 and `total` (inclusive).
	/// This function repeats the process up to `RandomAttemps` times if
	/// the number falls within the overflow range of the modulo operation to mitigate modulo bias.
	///
	/// Returns `None` if `total` is 0.
	fn random_number(total: u32) -> Option<u32> {
		if let Ok(seed) = Self::gen_random() {
			if total == 0 {
				return None
			}
			let mut random_number = Self::generate_random_number(seed);
			for _ in 1..T::RandomAttemps::get() {
				if random_number < u32::MAX.saturating_sub(u32::MAX.wrapping_rem(total)) {
					break
				}
				random_number = Self::generate_random_number(seed);
			}
			return Some((random_number.wrapping_rem(total)).saturating_add(1))
		}
		None
	}
}
