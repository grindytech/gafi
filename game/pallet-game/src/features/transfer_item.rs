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
		let from_balance = ItemBalances::<T, I>::get((who, collection, item));
		ensure!(
			amount <= from_balance,
			Error::<T, I>::InsufficientItemBalance
		);

		// update who's balance
		ItemBalances::<T, I>::insert((who, collection, item), from_balance - amount);

		// update destination's balance
		let to_balance = ItemBalances::<T, I>::get((destination, collection, item));
		ItemBalances::<T, I>::insert((destination, collection, item), to_balance + amount);

		Ok(())
	}

	fn swap() -> DispatchResult {
		todo!()
	}
}
