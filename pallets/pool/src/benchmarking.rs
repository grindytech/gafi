#![cfg(feature = "runtime-benchmarks")]

use crate::pool::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use crate::{Call, Config, Pallet};

const PACKS: [PackService; 3] = [PackService::Basic, PackService::Medium, PackService::Max];

benchmarks! {
	join_pool {
		let i in 0..3;
		let caller = account("caller", 0, 0);
	}: {
		Pallet::<T>::join(RawOrigin::Signed(caller).into(), PACKS[i as usize]);
	}

}

impl_benchmark_test_suite!(PalletPool, crate::mock::new_test_ext(), crate::mock::Test,);
