use super::*;

#[allow(unused)]
use crate::Pallet as GafiMembership;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use gafi_primitives::{constant::ID, system_services::SystemPool};

pub const UPFRONT_BASIC_ID: ID = [10_u8; 32];

benchmarks! {
	registration {
		let s in 1 .. 100;
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet::Config>::Currency::make_free_balance_be(&caller, <T as pallet::Config>::Currency::minimum_balance() * 10u32.into());
		assert!(<T as pallet::Config>::Currency::free_balance(&caller) > 0u32.into());

	}: _(RawOrigin::Signed(caller))

	remove_member {
		let s in 1 .. 100;
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet::Config>::Currency::make_free_balance_be(&caller, <T as pallet::Config>::Currency::minimum_balance() * 10u32.into());
		assert!(<T as pallet::Config>::Currency::free_balance(&caller) > 0u32.into());
		Pallet::<T>::registration(RawOrigin::Signed(caller.clone()).into());

	}: _(RawOrigin::Root, caller)

	impl_benchmark_test_suite!(GafiMembership, crate::mock::new_test_ext(), crate::mock::Test);
}
