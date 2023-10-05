use crate::{mock::*, Error, Event, RandomSeed, SeedPayload};
use frame_support::{assert_err, assert_noop, assert_ok, BoundedVec};
use gafi_support::game::GameRandomness;
use rand::Rng;

fn test_pub(seed: u8) -> sp_core::sr25519::Public {
	sp_core::sr25519::Public::from_raw([seed; 32])
}

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
fn random_number_should_works() {
	new_test_ext().execute_with(|| {
		let seed = [0_u8; 64];
		let random_seed = BoundedVec::<u8, SeedLength>::try_from(seed.to_vec());
		let payload = SeedPayload {
			block_number: 0_u64,
			seed: random_seed.unwrap(),
		};

		RandomSeed::<Test>::put(payload);

		let total = 10000;
		let attempts = 2;
		let mut values: Vec<u32> = vec![];

		for index in 0..attempts {
			let rand_number = OracleRandomness::random_number(total, index);
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
			OracleRandomness::set_new_random_urls(RuntimeOrigin::signed(test_pub(1)), urls),
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

#[test]
fn test_parse_randomness() {
	// Test case 1: Valid input with randomness value
	let result1 = r#"{"round":3366165,"randomness":"c25de9ba2cdf3ac9be2aa74dbf038aa6e84969151d51318946beafaf20f9c30b","signature":"9514585af8f888f54f6f6e784be0ccd973a354f1cee2f5c30077714a4c05392c6c051d53ebe76dcd10012cb011bec92100cab52101e46ad7fc8bd1ebdc8279ff1cac85a490aaf783ba6d3cf4658ae6d93a731b487bb046ab191abeb0c977478c","previous_signature":"939e6bd3fb386a847289ca00d10941915a05da184af69cc466c45f13a619126d5c941e5fe4d25dec0ed758dda8dbb41e06345a9f7e12191854daa5a9036b09685bd2c3fd69ea255eb38d66c7076966aab65a0954f13ad1f968da9bbe9bca689a"}"#;
	let expected1 = "c25de9ba2cdf3ac9be2aa74dbf038aa6e84969151d51318946beafaf20f9c30b"
		.as_bytes()
		.to_vec();

	assert_eq!(
		OracleRandomness::parse_randomness(result1).unwrap(),
		expected1
	);

	// Test case 2: Valid input without randomness value
	let result2 = r#"{"round":3366165,"signature":"9514585af8f888f54f6f6e784be0ccd973a354f1cee2f5c30077714a4c05392c6c051d53ebe76dcd10012cb011bec92100cab52101e46ad7fc8bd1ebdc8279ff1cac85a490aaf783ba6d3cf4658ae6d93a731b487bb046ab191abeb0c977478c","previous_signature":"939e6bd3fb386a847289ca00d10941915a05da184af69cc466c45f13a619126d5c941e5fe4d25dec0ed758dda8dbb41e06345a9f7e12191854daa5a9036b09685bd2c3fd69ea255eb38d66c7076966aab65a0954f13ad1f968da9bbe9bca689a"}"#;
	assert_eq!(OracleRandomness::parse_randomness(result2), None);

	// Test case 3: Invalid JSON input
	let result3 = r#"{"round":3366165,"randomness":"c25de9ba2cdf3ac9be2aa74dbf038aa6e84969151d51318946beafaf20f9c30b""signature":"9514585af8f888f54f6f6e784be0ccd973a354f1cee2f5c30077714a4c05392c6c051d53ebe76dcd10012cb011bec92100cab52101e46ad7fc8bd1ebdc8279ff1cac85a490aaf783ba6d3cf4658ae6d93a731b487bb046ab191abeb0c977478c","previous_signature":"939e6bd3fb386a847289ca00d10941915a05da184af69cc466c45f13a619126d5c941e5fe4d25dec0ed758dda8dbb41e06345a9f7e12191854daa5a9036b09685bd2c3fd69ea255eb38d66c7076966aab65a0954f13ad1f968da9bbe9bca689a"}"#;
	assert_eq!(OracleRandomness::parse_randomness(result3), None);
}
