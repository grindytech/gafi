//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as GameRandomness;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn submit_random_seed_unsigned() {
		let block_number = 1u32.into();
		let seed = [0; 32];
		#[extrinsic_call]
		submit_random_seed_unsigned(RawOrigin::None, block_number, seed);

		assert_eq!(RandomSeed::<T>::get().unwrap().seed, seed);
	}

	impl_benchmark_test_suite!(GameRandomness, crate::mock::new_test_ext(), crate::mock::Test);
}
