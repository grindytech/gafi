//! Benchmarking setup for upfront-pool

use super::*;
#[allow(unused)]
use crate::Pallet as Pool;
use crate::{Call, Config};
use frame_benchmarking::{benchmarks};
use frame_system::RawOrigin;

benchmarks! {
	set_max_player {
		let s in 0 .. 2;
	}: _(RawOrigin::Root, s)


	impl_benchmark_test_suite!(Pool, crate::mock::_new_test_ext(), crate::mock::Test);
}
