use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::tokens::nonfungibles_v2::{Create, Inspect},
};
use gafi_support::game::CreateCollection;
use pallet_nfts::{CollectionRole, CollectionRoles};
use sp_std::vec::Vec;

impl<T: Config<I>, I: 'static>
	CreateCollection<T::AccountId, T::GameId, T::CollectionId, CollectionConfigFor<T, I>>
	for Pallet<T, I>
{
	fn do_create_game_collection(
		who: &T::AccountId,
		game: &T::GameId,
		admin: &T::AccountId,
		config: &CollectionConfigFor<T, I>,
	) -> DispatchResult {
		// verify create collection role
		ensure!(
			GameRoleOf::<T, I>::get(game, &who) ==
				Some(CollectionRoles(
					CollectionRole::Issuer | CollectionRole::Freezer | CollectionRole::Admin
				)),
			Error::<T, I>::NoPermission
		);

		let collection = T::Nfts::create_collection(&who, &admin, &config);

		if let Ok(id) = collection {
			// insert game collections
			CollectionsOf::<T, I>::try_mutate(&game, |collection_vec| -> DispatchResult {
				collection_vec.try_push(id).map_err(|_| Error::<T, I>::ExceedMaxCollection)?;
				Ok(())
			})?;

			// insert collection game
			GameOf::<T, I>::insert(id, game);
			GameCollectionConfigOf::<T, I>::insert(id, config);
			Self::deposit_event(Event::<T, I>::CollectionCreated { collection: id });
		}
		Ok(())
	}

	fn do_create_collection(
		who: &T::AccountId,
		admin: &T::AccountId,
		config: &CollectionConfigFor<T, I>,
	) -> DispatchResult {
		let collection = T::Nfts::create_collection(&who, &admin, &config);
		if let Ok(id) = collection {
			GameCollectionConfigOf::<T, I>::insert(id, config);
			Self::deposit_event(Event::<T, I>::CollectionCreated { collection: id });
		}
		Ok(())
	}

	fn do_add_collection(
		who: &T::AccountId,
		game: &T::GameId,
		collection_ids: &Vec<T::CollectionId>,
	) -> DispatchResult {
		// make sure signer is game owner
		Self::ensure_game_owner(who, game)?;

		// make sure signer is collection owner
		for id in collection_ids {
			if let Some(owner) = T::Nfts::collection_owner(&id) {
				ensure!(owner == who.clone(), Error::<T, I>::NoPermission);
			} else {
				return Err(Error::<T, I>::UnknownCollection.into())
			}
		}

		CollectionsOf::<T, I>::try_mutate(&game, |collection_vec| -> DispatchResult {
			collection_vec
				.try_extend(collection_ids.clone().into_iter())
				.map_err(|_| <Error<T, I>>::ExceedMaxCollection)?;
			Ok(())
		})?;

		for id in collection_ids {
			GameOf::<T, I>::insert(id, game);
		}

		Ok(())
	}
}
