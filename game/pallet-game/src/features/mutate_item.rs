use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::game::{Amount, MutateItem};

impl<T: Config<I>, I: 'static> MutateItem<T::AccountId, T::GameId, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{
	/// Burns a specified amount of an item from an account's balance and decreases the finite
	/// supply.
	///
	/// Emits `Burned` event on success.
	fn do_burn(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		Self::sub_item_balance(who, collection, item, amount)?;

		Self::decrease_finite_item_supply(collection, item, amount);

		Self::deposit_event(Event::<T, I>::Burned {
			who: who.clone(),
			collection: *collection,
			item: *item,
			amount,
		});
		Ok(())
	}
}
