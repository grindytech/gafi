use crate::*;
use frame_support::{pallet_prelude::*, traits::tokens::nonfungibles_v2::Create};
use gafi_support::game::CreateCollection;
use pallet_nfts::{CollectionRole, CollectionRoles};

impl<T: Config<I>, I: 'static>
	CreateCollection<T::AccountId, T::GameId, T::CollectionId, CollectionConfigFor<T, I>>
	for Pallet<T, I>
{
	fn do_create_game_collection(
		who: &T::AccountId,
		game: &T::GameId,
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
		if let Some(game_details) = Game::<T, I>::get(game) {
			let collection = T::Nfts::create_collection(&game_details.owner, &who, &config);

			if let Ok(id) = collection {
				// insert game collections
				CollectionsOf::<T, I>::try_mutate(&game, |collection_vec| -> DispatchResult {
					collection_vec.try_push(id).map_err(|_| Error::<T, I>::ExceedMaxCollection)?;
					Ok(())
				})?;

				// insert collection game
				GameOf::<T, I>::insert(id, game);
				GameCollectionConfigOf::<T, I>::insert(id, config);
				Self::deposit_event(Event::<T, I>::CollectionCreated {
					who: who.clone(),
					collection: id,
				});
				return Ok(())
			}
		}
		Err(Error::<T, I>::UnknownGame.into())
	}

	fn do_create_collection(
		who: &T::AccountId,
		admin: &T::AccountId,
		config: &CollectionConfigFor<T, I>,
	) -> DispatchResult {
		let collection = T::Nfts::create_collection(&who, &admin, &config);
		if let Ok(id) = collection {
			GameCollectionConfigOf::<T, I>::insert(id, config);
			Self::deposit_event(Event::<T, I>::CollectionCreated {
				who: who.clone(),
				collection: id,
			});
		}
		Ok(())
	}

	fn do_add_collection(
		who: &T::AccountId,
		game: &T::GameId,
		collection: &T::CollectionId,
	) -> DispatchResult {
		// make sure signer is game owner
		Self::ensure_game_owner(who, game)?;

		// make sure signer is collection owner
		Self::ensure_collection_owner(who, collection)?;

		CollectionsOf::<T, I>::try_mutate(&game, |collection_vec| -> DispatchResult {
			ensure!(
				!collection_vec.contains(collection),
				Error::<T, I>::CollectionExists
			);

			collection_vec
				.try_push(*collection)
				.map_err(|_| <Error<T, I>>::ExceedMaxCollection)?;
			Ok(())
		})?;

		GameOf::<T, I>::insert(collection, game);
		Ok(())
	}
}
