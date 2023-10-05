//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as OracleRandomness;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn submit_random_seed_unsigned() {
		let block_number = 1u32.into();
		let seed = [0u8; 64].to_vec();

		#[extrinsic_call]
		submit_random_seed_unsigned(RawOrigin::None, block_number, seed);
	}

	#[benchmark]
	fn set_new_random_urls() {
		let urls = vec![
			"https://api2.drand.sh/public/latest".as_bytes().to_vec(),
			"https://api.drand.sh/public/latest".as_bytes().to_vec(),
			"https://api.drand.sh/public/latest".as_bytes().to_vec(),
			"https://api.drand.sh/public/latest".as_bytes().to_vec(),
		];

		#[extrinsic_call]
		set_new_random_urls(RawOrigin::Root, urls);
	}

	impl_benchmark_test_suite!(
		OracleRandomness,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}
