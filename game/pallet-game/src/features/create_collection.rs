use crate::{types::GameDetails, *};
use frame_support::{
	pallet_prelude::*,
	traits::tokens::nonfungibles_v2::{Create as NftsCreate, Inspect},
};
use gafi_support::{
	common::{BlockNumber, Hash},
	game::{Amount, Create},
};
use pallet_nfts::{CollectionConfig, CollectionRole, CollectionRoles};

impl<T: Config<I>, I: 'static>
	Create<T::AccountId, T::GameId, T::CollectionId, T::ItemId, CollectionConfigFor<T, I>, ItemConfig>
	for Pallet<T, I>
{
	fn do_create_game_collection(
		who: T::AccountId,
		game_id: T::GameId,
		maybe_admin: Option<T::AccountId>,
		config: CollectionConfigFor<T, I>,
	) -> DispatchResult {
		// verify create collection role
		ensure!(
			GameRoleOf::<T, I>::get(game_id, &who) ==
				Some(CollectionRoles(
					CollectionRole::Issuer | CollectionRole::Freezer | CollectionRole::Admin
				)),
			Error::<T, I>::NoPermission
		);

		// get admin or owner is an admin in default
		let admin = match maybe_admin {
			Some(ad) => ad,
			None => who.clone(),
		};
		let collection_id = T::Nfts::create_collection(&who, &admin, &config);

		if let Ok(id) = collection_id {
			let _result = GameCollections::<T, I>::try_mutate(&game_id, |collection_vec| {
				collection_vec.try_push(id)
			})
			.map_err(|_| <Error<T, T>>::ExceedMaxCollection);
			Self::deposit_event(Event::<T, I>::CollectionCreated { id });
		}
		Ok(())
	}

	fn do_create_collection(
		who: T::AccountId,
		maybe_admin: Option<T::AccountId>,
		config: CollectionConfigFor<T, I>,
	) -> DispatchResult {
		let admin = match maybe_admin {
			Some(ad) => ad,
			None => who.clone(),
		};
		let collection_id = T::Nfts::create_collection(&who, &admin, &config);
		if let Ok(id) = collection_id {
			Self::deposit_event(Event::<T, I>::CollectionCreated { id });
		}
		Ok(())
	}

	fn do_create_item(
		who: T::AccountId,
		collection_id: T::CollectionId,
		item_id: T::ItemId,
		item_config: ItemConfig,
		amount: Amount,
	) -> DispatchResult {
		Ok(())
	}

	fn do_add_item(
		who: T::AccountId,
		collection_id: T::CollectionId,
		item_id: T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		Ok(())
	}
}
