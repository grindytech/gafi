use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::game::{Bundle, Distribution, Mining};

impl<T: Config<I>, I: 'static>
	Mining<T::AccountId, BalanceOf<T, I>, T::CollectionId, T::ItemId, T::PoolId> for Pallet<T, I>
{
	fn do_create_dynamic_pool(
		pool: &T::PoolId,
		who: &T::AccountId,
		resource: Bundle<T::CollectionId, T::ItemId>,
		fee: BalanceOf<T, I>,
	) -> DispatchResult {
		// ensure pool is available

		// Deposit balance

		// Reserve item balance

		// create new pool

		// fee

		todo!()
	}

	fn do_create_stable_pool(
		pool: &T::PoolId,
		who: &T::AccountId,
		distribution: Distribution<T::CollectionId, T::ItemId>,
		fee: BalanceOf<T, I>,
	) -> DispatchResult {
		todo!()
	}
}
