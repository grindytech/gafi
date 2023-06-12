use crate::{*};
use frame_support::pallet_prelude::*;
use gafi_support::game::{Mining, Distribution, Bundle};

impl<T: Config<I>, I: 'static> Mining<T::AccountId, BalanceOf<T, I>, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{
	fn do_create_dynamic_pool(
		who: &T::AccountId,
		pool: Bundle<T::CollectionId, T::ItemId>,
		fee: BalanceOf<T, I>,
	) -> DispatchResult {
		todo!()
	}

	fn do_create_stable_pool(
		who: &T::AccountId,
		distribution: Distribution<T::CollectionId, T::ItemId>,
		fee: BalanceOf<T, I>,
	) -> DispatchResult {
		todo!()
	}
}
