use crate::*;
use frame_support::{pallet_prelude::*, StorageValue};
use pallet_nfts::{CollectionRole, CollectionRoles};

impl<T: Config<I>, I: 'static> GameSetting<T::AccountId, T::GameId, T::StringLimit>
	for Pallet<T, I>
{
	fn do_set_game_metadata(
		maybe_check_origin: Option<T::AccountId>,
		game: T::GameId,
		data: BoundedVec<u8, T::StringLimit>,
	) -> DispatchResult {
		let game_details = Game::<T, I>::get(game).ok_or(Error::<T, I>::UnknownGame)?;

		if let Some(check_origin) = &maybe_check_origin {
			ensure!(
				game_details.admin == *check_origin,
				Error::<T, I>::NoPermission
			);
		}

		let is_root = maybe_check_origin.is_none();

		GameMetadataOf::<T, I>::try_mutate_exists(game, |metadata| {
			*metadata = Some(GameMetadata { data: data.clone() });
			Self::deposit_event(Event::GameSetMetadata {
				who: maybe_check_origin,
				game,
				data,
			});
			Ok(())
		})
	}

	fn do_create_game(
		game: &T::GameId,
		who: &T::AccountId,
		admin: &T::AccountId,
	) -> DispatchResult {
		<T as Config<I>>::Currency::reserve(who, T::GameDeposit::get())?;

		let details = GameDetails {
			owner: who.clone(),
			collections: 0,
			owner_deposit: T::GameDeposit::get(),
			admin: admin.clone(),
		};

		GameRoleOf::<T, I>::insert(
			game,
			admin,
			CollectionRoles(
				CollectionRole::Admin | CollectionRole::Freezer | CollectionRole::Issuer,
			),
		);

		GameAccount::<T, I>::insert(who, game, ());
		Game::<T, I>::insert(game, details);
		Self::deposit_event(Event::GameCreated {
			who: who.clone(),
			game: *game,
		});
		Ok(())
	}
}
