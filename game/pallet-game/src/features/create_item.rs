use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::tokens::nonfungibles_v2::{Create, Inspect},
};
use gafi_support::game::{Amount, CreateItem};
use pallet_nfts::{CollectionRole, CollectionRoles};
use sp_core::TryCollect;

impl<T: Config<I>, I: 'static> CreateItem<T::AccountId, T::CollectionId, T::ItemId, ItemConfig>
	for Pallet<T, I>
{
	fn do_create_item(
		who: T::AccountId,
		collection_id: T::CollectionId,
		item_id: T::ItemId,
		config: ItemConfig,
		amount: Amount,
	) -> DispatchResult {
		// ensure permission
		if let Some(game_id) = CollectionGame::<T, I>::get(collection_id) {
			ensure!(
				GameRoleOf::<T, I>::get(game_id, &who) ==
					Some(CollectionRoles(
						CollectionRole::Issuer | CollectionRole::Freezer | CollectionRole::Admin
					)),
				Error::<T, I>::NoPermission
			);

			T::Nfts::mint_into(&collection_id, &item_id, &who, &config, false)?;

			let _result = ItemReserve::<T, I>::try_mutate(&collection_id, |reserve_vec| {
				reserve_vec.try_push((item_id, amount))
			})
			.map_err(|_| <Error<T, T>>::ExceedMaxItem);

			Self::deposit_event(Event::<T, I>::ItemCreated {
				collection_id,
				item_id,
				amount,
			});
			Ok(())
		} else {
			return Err(Error::<T, I>::UnknownCollection.into())
		}
	}

	fn do_add_item(
		who: T::AccountId,
		collection_id: T::CollectionId,
		item_id: T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		// ensure permission
		if let Some(game_id) = CollectionGame::<T, I>::get(collection_id) {
			ensure!(
				GameRoleOf::<T, I>::get(game_id, &who) ==
					Some(CollectionRoles(
						CollectionRole::Issuer | CollectionRole::Freezer | CollectionRole::Admin
					)),
				Error::<T, I>::NoPermission
			);

			let result = ItemReserve::<T, I>::try_mutate(&collection_id, |reserve_vec| {
				let balances = reserve_vec.into_mut();
				for balance in balances {
					if balance.0 == item_id {
						balance.1 += amount;
						return Ok(balance.1)
					}
				}
				return Err(Error::<T, I>::UnknownItem)
			})
			.map_err(|err| err);

			Self::deposit_event(Event::<T, I>::ItemCreated {
				collection_id,
				item_id,
				amount: result.unwrap_or_default(),
			});
		} else {
			return Err(Error::<T, I>::UnknownCollection.into())
		}

		Ok(())
	}
}
