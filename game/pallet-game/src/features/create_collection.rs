use crate::{types::GameDetails, *};
use frame_support::pallet_prelude::*;
use gafi_support::game::{Amount, Create, CollectionId};
use gafi_support::common::{Hash, BlockNumber};
use frame_support::traits::tokens::nonfungibles_v2::Create as NftsCreate;
use pallet_nfts::{CollectionConfig};

impl<T: Config<I>, I: 'static> Create<T::AccountId, T::GameId, T::CollectionId, T::ItemId, CollectionConfigFor<T, I>>
	for Pallet<T, I>
	// where T::CollectionId: From<u32> + From<(Hash, BlockNumber)>, 
	// where T::CollectionId: Into(CollectionId), 
{
	fn do_create_game_collection(
		game_id: T::GameId,
		who: T::AccountId,
		admin: T::AccountId,
		config: CollectionConfigFor<T, I>,
	) -> DispatchResult {
		T::Nfts::create_collection(&who, &admin, &config);

		Ok(())
	}

	fn do_create_collection(
		collection_id: T::CollectionId,
		owner: T::AccountId,
		admin: T::AccountId,
	) -> DispatchResult {
		Ok(())
	}

	fn do_create_item(
		collection_id: T::CollectionId,
		item_id: T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		Ok(())
	}

	fn do_add_item(
		collection_id: T::CollectionId,
		item_id: T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		Ok(())
	}
}
