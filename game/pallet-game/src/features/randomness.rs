/// Item module provides utility functions for pallet-game
use crate::*;
use frame_support::pallet_prelude::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
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

	/// Generates a random number within the range [1, total] with reduced bias, based on the
	/// provided `seed`. The function uses the `seed` to generate a random number and repeats the
	/// process up to `random_attempts` times if the number falls within the overflow range of the
	/// modulo operation to mitigate modulo bias. If `total` is 0 or `random_attempts` is 0, the
	/// function returns `None`.
	///
	/// # Arguments
	///
	/// * `total` - The upper limit for the random number (inclusive). Must be greater than or equal
	///   to 1.
	/// * `seed` - A seed value used for generating random numbers.
	/// * `random_attempts` - The maximum number of attempts to generate a random number.
	///
	/// # Returns
	///
	/// * `Some(random_number)` - If `total` is greater than or equal to 1 and `random_attempts` is
	///   greater than 0, it returns a random number within the range [1, total].
	/// * `None` - If `total` is 0 or `random_attempts` is 0, or if no valid random number is
	///   generated after `random_attempts` attempts.
	///
	/// # Note
	///
	/// The `random_attempts` parameter can be adjusted to trade off between bias reduction and
	/// performance. Higher values of `random_attempts` may provide better bias reduction but may
	/// also lead to longer execution times.
	pub(crate) fn random_number(total: u32, seed: u32, random_attemps: u32) -> Option<u32> {
		if total == 0 || random_attemps == 0 {
			return None
		}

		let mut random_number = Self::generate_random_number(seed);
		for _ in 1..random_attemps {
			if random_number < u32::MAX.saturating_sub(u32::MAX % total) {
				break
			}

			random_number = Self::generate_random_number(seed);
		}

		Some(random_number % total + 1)
	}

	/// Generate a random number from the off-chain worker's random seed
	pub(crate) fn gen_random() -> u32 {
		let seed = RandomSeed::<T, I>::get();

		let random = <u32>::decode(&mut TrailingZeroInput::new(seed.as_ref()))
			.expect("input is padded with zeroes; qed");

		random
	}
}
