use crate::{*};
use frame_support::{
	pallet_prelude::*,
	traits::tokens::nonfungibles_v2::{Create, Inspect},
};
use gafi_support::{
	game::{CreateCollection},
};
use pallet_nfts::{CollectionRole, CollectionRoles};
use sp_std::vec::Vec;

impl<T: Config<I>, I: 'static>
	CreateCollection<T::AccountId, T::GameId, T::CollectionId, CollectionConfigFor<T, I>>
	for Pallet<T, I>
{
	fn do_create_game_collection(
		who: &T::AccountId,
		game_id: &T::GameId,
		admin: &T::AccountId,
		config: &CollectionConfigFor<T, I>,
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

		let collection_id =
			T::Nfts::create_collection(&who, &admin, &config.to_collection_config());

		if let Ok(id) = collection_id {
			// insert game collections
			let _result = GameCollections::<T, I>::try_mutate(&game_id, |collection_vec| {
				collection_vec.try_push(id)
			})
			.map_err(|_| <Error<T, T>>::ExceedMaxCollection);

			// insert collection game
			CollectionGame::<T, I>::insert(id, game_id);
			GameCollectionConfigOf::<T, I>::insert(id, config);
			Self::deposit_event(Event::<T, I>::CollectionCreated { collection_id: id });
		}
		Ok(())
	}

	fn do_create_collection(
		who: &T::AccountId,
		admin: &T::AccountId,
		config: &CollectionConfigFor<T, I>,
	) -> DispatchResult {
		let collection_id =
			T::Nfts::create_collection(&who, &admin, &config.to_collection_config());
		if let Ok(id) = collection_id {
			GameCollectionConfigOf::<T, I>::insert(id, config);
			Self::deposit_event(Event::<T, I>::CollectionCreated { collection_id: id });
		}
		Ok(())
	}

	fn do_add_collection(
		who: &T::AccountId,
		game_id: &T::GameId,
		collection_ids: &Vec<T::CollectionId>,
	) -> DispatchResult {
		// make sure signer is game owner
		if let Some(game) = Games::<T, I>::get(game_id) {
			ensure!(game.owner == who.clone(), Error::<T, I>::NoPermission);
		} else {
			return Err(Error::<T, I>::UnknownGame.into())
		}

		// make sure signer is collection owner
		for id in collection_ids {
			if let Some(owner) = T::Nfts::collection_owner(&id) {
				ensure!(owner == who.clone(), Error::<T, I>::NoPermission);
			} else {
				return Err(Error::<T, I>::UnknownCollection.into())
			}
		}

		let _result = GameCollections::<T, I>::try_mutate(&game_id, |collection_vec| {
			collection_vec.try_extend(collection_ids.clone().into_iter())
		})
		.map_err(|_| <Error<T, T>>::ExceedMaxCollection);

		for id in collection_ids {
			CollectionGame::<T, I>::insert(id, game_id);
		}

		Ok(())
	}
}
