use crate::*;
use frame_support::pallet_prelude::*;
use pallet_nfts::{CollectionRole, CollectionRoles};

impl<T: Config<I>, I: 'static> GameSetting<T::AccountId, T::GameId>
	for Pallet<T, I>
{
	// SBP-M2: `who` is a reference type, no need to pass it's reference again in inner calls.
	// SBP-M2: Please make sure to handle duplicate record error.
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

		GameAccount::<T, I>::insert( who, game, ());
		Game::<T, I>::insert(game, details);
		Self::deposit_event(Event::GameCreated { who: who.clone(), game: *game });
		Ok(())
	}
}
