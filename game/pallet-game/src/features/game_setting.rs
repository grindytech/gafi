use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::common::{BlockNumber};
use pallet_nfts::{CollectionRoles, CollectionRole};
use sp_runtime::Percent;


impl<T: Config<I>, I: 'static> GameSetting<T::AccountId, T::GameId> for Pallet<T, I> {
    fn do_create_game(
        id: T::GameId,
        owner: T::AccountId,
        maybe_admin: Option<T::AccountId>,
    ) -> DispatchResult {
		<T as Config<I>>::Currency::reserve(&owner, T::GameDeposit::get())?;

        let game= GameDetails {
			owner: owner,
			collections: 0,
			owner_deposit: T::GameDeposit::get(),
		};
		let next_id = id.increment();
        NextGameId::<T, I>::set(Some(next_id));

        if let Some(admin) = maybe_admin {
            GameRoleOf::<T, I>::insert(
                id,
                admin,
                CollectionRoles(
                    CollectionRole::Admin | CollectionRole::Freezer | CollectionRole::Issuer,
                ),
            );
        }

		Games::<T, I>::insert(id, game);
        Self::deposit_event(Event::GameCreated { id });
        Ok(())
    }

    fn do_set_swap_fee(
        id: T::GameId,
        owner: T::AccountId,
        fee: Percent,
        start_block: BlockNumber,
    ) -> DispatchResult {

        Ok(())
    }
}
