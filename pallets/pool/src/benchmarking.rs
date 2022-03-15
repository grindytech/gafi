#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use crate::mock::{SERVICES};

benchmarks!{

    join_pool {
        let b in SERVICES;
        let caller = account("caller", 0, 0);
    }: join(RawOrigin::Signed(caller), b);

}


impl_benchmark_test_suite!(
    PalletPool,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);