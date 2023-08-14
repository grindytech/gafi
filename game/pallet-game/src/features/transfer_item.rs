use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::game::{Amount, TransferItem};

impl<T: Config<I>, I: 'static> TransferItem<T::AccountId, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{
	/// Transfers a specified amount of an item from one account to another within a collection.
	///
	/// Emits `Transferred` event on success.
	fn do_transfer_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		destination: &T::AccountId,
		amount: Amount,
	) -> DispatchResult {
		Self::transfer_item(who, collection, item, destination, amount)?;

		Self::deposit_event(Event::<T, I>::Transferred {
			from: who.clone(),
			collection: *collection,
			item: *item,
			dest: destination.clone(),
			amount,
		});
		Ok(())
	}
}
