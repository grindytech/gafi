use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::game::{Amount, TransferItem};

impl<T: Config<I>, I: 'static> TransferItem<T::AccountId, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{
	fn do_transfer_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		destination: &T::AccountId,
		amount: Amount,
	) -> DispatchResult {
		
		Self::minus_item_balance(who, collection, item, amount)?;
		Self::add_item_balance(destination, collection, item, amount)?;

		Self::deposit_event(Event::<T, I>::Transferred {
			from: who.clone(),
			collection_id: *collection,
			item_id: *item,
			dest: destination.clone(),
			amount,
		});
		Ok(())
	}

	fn swap() -> DispatchResult {
		todo!()
	}
}
