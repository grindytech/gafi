use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::{common::{BlockNumber, AccountId}, game::{Create, Amount}};
use pallet_nfts::CollectionConfig;

impl<T: Config<I>, I: 'static> Create<T::GameId, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{
	fn create_game_collection(
		game_id: T::GameId,
		collection_id: T::CollectionId,
		owner: AccountId,
		admin: AccountId,
	) -> DispatchResult {
		Ok(())
	}

	fn create_collection(
		collection_id: T::CollectionId,
		owner: AccountId,
		admin: AccountId,
	) -> DispatchResult {
		Ok(())
	}

	fn create_item(
		collection_id: T::CollectionId,
		item_id: T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		Ok(())
	}

	fn add_item(collection_id: T::CollectionId, item_id: T::ItemId, amount: Amount) -> DispatchResult {
		Ok(())
	}
}
