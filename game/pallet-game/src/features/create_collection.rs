use crate::{*, types::GameDetails};
use frame_support::pallet_prelude::*;
use gafi_support::{game::{Create, Amount}};

impl<T: Config<I>, I: 'static> Create<T::AccountId, T::GameId, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{
	fn create_game_collection(
		game_id: T::GameId,
		collection_id: T::CollectionId,
		owner: T::AccountId,
		admin: T::AccountId,
	) -> DispatchResult {

		Ok(())
	}

	fn create_collection(
		collection_id: T::CollectionId,
		owner: T::AccountId,
		admin: T::AccountId,
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
