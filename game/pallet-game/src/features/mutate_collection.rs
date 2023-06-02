use crate::*;
use frame_support::{pallet_prelude::*, traits::tokens::nonfungibles_v2::Create};
use gafi_support::game::MutateCollection;
use pallet_nfts::{CollectionRole, CollectionRoles, CollectionSettings, MintSettings};

impl<T: Config<I>, I: 'static>
	MutateCollection<
		T::AccountId,
		T::GameId,
		T::CollectionId,
		CollectionConfigFor<T, I>,
		BalanceOf<T, I>,
	> for Pallet<T, I>
{
	fn do_create_game_collection(
		who: &T::AccountId,
		game: &T::GameId,
		fee: BalanceOf<T, I>,
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
			let config: CollectionConfigFor<T, I> = CollectionConfig {
				settings: CollectionSettings::default(),
				max_supply: None,
				mint_settings: MintSettings::default(),
			};

			let maybe_collection = T::Nfts::create_collection(&game_details.owner, &who, &config);

			if let Ok(collection) = maybe_collection {
				// insert fee
				MintingFeeOf::<T, I>::insert(collection, fee);

				// insert game collections
				CollectionsOf::<T, I>::try_mutate(&game, |collection_vec| -> DispatchResult {
					collection_vec
						.try_push(collection)
						.map_err(|_| Error::<T, I>::ExceedMaxCollection)?;
					Ok(())
				})?;

				// insert collection game
				GameOf::<T, I>::insert(collection, game);
				Self::deposit_event(Event::<T, I>::CollectionCreated {
					who: who.clone(),
					collection,
				});
				return Ok(())
			}
		}
		Err(Error::<T, I>::UnknownGame.into())
	}

	fn do_create_collection(
		who: &T::AccountId,
		admin: &T::AccountId,
		fee: BalanceOf<T, I>,
	) -> DispatchResult {
		let config: CollectionConfigFor<T, I> = CollectionConfig {
			settings: CollectionSettings::default(),
			max_supply: None,
			mint_settings: MintSettings::default(),
		};

		let maybe_collection = T::Nfts::create_collection(&who, &admin, &config);
		if let Ok(collection) = maybe_collection {
				// insert fee
				MintingFeeOf::<T, I>::insert(collection, fee);
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

	fn do_remove_collection(
		who: &T::AccountId,
		game: &T::GameId,
		collection: &T::CollectionId,
	) -> DispatchResult {
		// make sure signer is game owner
		Self::ensure_game_owner(who, game)?;
		// make sure signer is collection owner
		Self::ensure_collection_owner(who, collection)?;

		CollectionsOf::<T, I>::try_mutate(&game, |collection_vec| -> DispatchResult {
			let maybe_position = collection_vec.iter().position(|x| *x == *collection);
			match maybe_position {
				Some(position) => {
					collection_vec.remove(position);
					Ok(())
				},
				None => return Err(Error::<T, I>::UnknownCollection.into()),
			}
		})?;
		GameOf::<T, I>::remove(collection);
		Self::deposit_event(Event::<T,I>::CollectionRemoved {
			who: who.clone(),
			game: *game,
			collection: *collection,
		});
		Ok(())
	}
}
