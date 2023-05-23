use crate::*;
use frame_support::pallet_prelude::*;
use pallet_nfts::{CollectionRole, CollectionRoles};
use sp_runtime::Percent;

impl<T: Config<I>, I: 'static> GameSetting<T::AccountId, T::GameId, T::BlockNumber>
	for Pallet<T, I>
{
	fn do_create_game(
		who: &T::AccountId,
		id: &T::GameId,
		admin: &T::AccountId,
	) -> DispatchResult {
		<T as Config<I>>::Currency::reserve(&who, T::GameDeposit::get())?;

		let game = GameDetails {
			owner: who.clone(),
			collections: 0,
			owner_deposit: T::GameDeposit::get(),
			admin: admin.clone(),
		};
		let next_id = id.increment();
		NextGameId::<T, I>::set(Some(next_id));

		GameRoleOf::<T, I>::insert(
			id,
			admin,
			CollectionRoles(
				CollectionRole::Admin | CollectionRole::Freezer | CollectionRole::Issuer,
			),
		);

		Game::<T, I>::insert(id, game);
		Self::deposit_event(Event::GameCreated { game_id: *id });
		Ok(())
	}
}
