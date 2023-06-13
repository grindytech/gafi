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
				// insert game collections
				CollectionsOf::<T, I>::try_mutate(&game, |collection_vec| -> DispatchResult {
					collection_vec
						.try_push(collection)
						.map_err(|_| Error::<T, I>::ExceedMaxCollection)?;
					Ok(())
				})?;

				// insert collection game
				GamesOf::<T, I>::try_mutate(collection, |game_vec| -> DispatchResult {
					game_vec.try_push(*game).map_err(|_| Error::<T, I>::ExceedMaxGameShare)?;
					Ok(())
				})?;

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
	) -> DispatchResult {
		let config: CollectionConfigFor<T, I> = CollectionConfig {
			settings: CollectionSettings::default(),
			max_supply: None,
			mint_settings: MintSettings::default(),
		};

		let maybe_collection = T::Nfts::create_collection(&who, &admin, &config);
		if let Ok(collection) = maybe_collection {
			Self::deposit_event(Event::<T, I>::CollectionCreated {
				who: who.clone(),
				collection,
			});
			return	Ok(())
		}
		Err(Error::<T, I>::UnknownCollection.into())
	}

	fn do_set_accept_adding(
		who: &T::AccountId,
		game: &T::GameId,
		collection: &T::CollectionId,
	) -> DispatchResult {
		if let Some(collection_owner) = T::Nfts::collection_owner(collection) {
			ensure!(
				T::Nfts::is_admin(collection, who),
				Error::<T, I>::NoPermission
			);
			<T as Config<I>>::Currency::reserve(&collection_owner, T::GameDeposit::get())?;
			AddingAcceptance::<T, I>::insert(collection, game);

			Self::deposit_event(Event::<T, I>::AddingAcceptanceSet {
				who: who.clone(),
				game: *game,
				collection: *collection,
			});
			return Ok(())
		}
		Err(Error::<T, I>::UnknownCollection.into())
	}

	fn do_add_collection(
		who: &T::AccountId,
		game: &T::GameId,
		collection: &T::CollectionId,
	) -> DispatchResult {
		// make sure signer is game owner
		ensure!(
			Self::has_role(game, who, CollectionRole::Admin),
			Error::<T, I>::NoPermission
		);

		match AddingAcceptance::<T, I>::get(collection) {
			Some(id) => {
				ensure!(id == *game, Error::<T, I>::UnknownAcceptance);
			},
			None => return Err(Error::<T, I>::UnknownAcceptance.into()),
		};

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

		// insert collection game
		GamesOf::<T, I>::try_mutate(collection, |game_vec| -> DispatchResult {
			game_vec.try_push(*game).map_err(|_| Error::<T, I>::ExceedMaxGameShare)?;
			Ok(())
		})?;

		Self::deposit_event(Event::<T, I>::CollectionAdded {
			who: who.clone(),
			game: *game,
			collection: *collection,
		});

		Ok(())
	}

	fn do_remove_collection(
		who: &T::AccountId,
		game: &T::GameId,
		collection: &T::CollectionId,
	) -> DispatchResult {
		ensure!(
			T::Nfts::is_admin(collection, who) | Self::has_role(game, who, CollectionRole::Admin),
			Error::<T, I>::NoPermission
		);

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
		GamesOf::<T, I>::remove(collection);
		Self::deposit_event(Event::<T, I>::CollectionRemoved {
			who: who.clone(),
			game: *game,
			collection: *collection,
		});
		Ok(())
	}
}
