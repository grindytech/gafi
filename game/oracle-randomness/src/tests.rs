use crate::{mock::*, Error, Event};
use frame_support::{assert_err, assert_noop, assert_ok};
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

#[test]
fn set_new_random_urls_works() {
	new_test_ext().execute_with(|| {
		// Set up
		let urls = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];

		// Ensure root origin
		System::set_block_number(1);
		assert_ok!(OracleRandomness::set_new_random_urls(
			RuntimeOrigin::root(),
			urls.clone(),
		));

		// Verify storage
		assert_eq!(OracleRandomness::urls().to_vec(), urls);

		// Ensure non-root origin fails
		assert_err!(
			OracleRandomness::set_new_random_urls(RuntimeOrigin::signed(1), urls),
			frame_support::error::BadOrigin
		);

		// Ensure exceeding max random URL length fails
		let long_urls = vec![
			vec![1; (URL_LENGTH + 1) as usize],
			vec![2; (URL_LENGTH + 1) as usize],
		];
		assert_err!(
			OracleRandomness::set_new_random_urls(RuntimeOrigin::root(), long_urls),
			Error::<Test>::ExceedRandomURLLength
		);

		// Ensure exceeding max random URL count fails
		let too_many_urls = vec![vec![1; URL_LENGTH as usize]; (MAX_RANDOM_URL + 1) as usize];
		assert_err!(
			OracleRandomness::set_new_random_urls(RuntimeOrigin::root(), too_many_urls),
			Error::<Test>::ExceedMaxRandomURL
		);
	});
}
