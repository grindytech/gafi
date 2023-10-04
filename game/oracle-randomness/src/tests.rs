use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use rand::Rng;

#[test]
fn gen_random_should_works() {
	new_test_ext().execute_with(|| {
		let mut rng = rand::thread_rng();
		let mut seeds: Vec<Vec<u8>> = vec![];

		let max_seed = 10;
		for _ in 0..max_seed {
			let seed: [u8; 32] = rng.gen();
			seeds.push(seed.to_vec())
		}

		let mut values: Vec<u32> = vec![];

		let total = 10000;
		let attempts = 5;
		for seed in seeds {
			let rand_number = OracleRandomness::random_bias(&seed, total, attempts);
			values.push(rand_number.unwrap());
		}

		let all_lower_or_equal = values.iter().all(|&value| value <= total);
		let not_all_equal = values.iter().any(|&value| value != values[0]);

		assert!(all_lower_or_equal);
		assert!(not_all_equal);
	});
}
