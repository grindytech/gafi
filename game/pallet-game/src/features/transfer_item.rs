use crate::{types::Item, *};
use frame_support::{
	pallet_prelude::*,
	traits::tokens::nonfungibles_v2::{Create, Inspect},
};
use gafi_support::game::{Amount, CreateItem, TransferItem};
use pallet_nfts::{CollectionRole, CollectionRoles};
use sp_core::TryCollect;

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
		let from_balance = ItemBalances::<T, I>::get((collection, who, item));
		ensure!(
			amount <= from_balance,
			Error::<T, I>::InsufficientItemBalance
		);

		// update who's balance
		ItemBalances::<T, I>::insert((collection, who, item), from_balance - amount);

		// update destination's balance
		let to_balance = ItemBalances::<T, I>::get((collection, destination, item));
		ItemBalances::<T, I>::insert((collection, destination, item), to_balance + amount);

		Ok(())
	}

	fn swap() -> DispatchResult {
		todo!()
	}
}
