use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, MutateItem};

impl<T: Config<I>, I: 'static> MutateItem<T::AccountId, T::GameId, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{
	fn do_burn(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		Self::sub_item_balance(who, collection, item, amount)?;

		Self::deposit_event(Event::<T, I>::Burned {
			who: who.clone(),
			collection: *collection,
			item: *item,
			amount,
		});
		Ok(())
	}
}
