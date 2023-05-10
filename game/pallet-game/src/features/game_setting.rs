use crate::*;
use frame_support::pallet_prelude::*;
use pallet_nfts::{CollectionRole, CollectionRoles};
use sp_runtime::Percent;

impl<T: Config<I>, I: 'static> GameSetting<T::AccountId, T::GameId, T::BlockNumber>
	for Pallet<T, I>
{
	fn do_create_game(
		id: T::GameId,
		who: T::AccountId,
		maybe_admin: Option<T::AccountId>,
	) -> DispatchResult {
		<T as Config<I>>::Currency::reserve(&who, T::GameDeposit::get())?;

		let admin = match maybe_admin {
			Some(admin) => admin,
			None => who.clone(),
		};

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

		Games::<T, I>::insert(id, game);
		Self::deposit_event(Event::GameCreated { id });
		Ok(())
	}

	fn do_set_swap_fee(
		id: T::GameId,
		who: T::AccountId,
		fee: Percent,
		start_block: BlockNumber<T>,
	) -> DispatchResult {
		ensure!(
			Self::has_role(&id, &who, CollectionRole::Admin),
			Error::<T, I>::NoPermission
		);

		ensure!(fee <= T::MaxSwapFee::get(), Error::<T, I>::SwapFeeTooHigh);

		SwapFee::<T, I>::insert(id, (fee, start_block));
		Self::deposit_event(Event::SwapFeeSetted { id, fee });
		Ok(())
	}
}
