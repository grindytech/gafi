use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::tokens::nonfungibles_v2::{Create, Inspect},
};
use gafi_support::game::{Amount, CreateItem};
use pallet_nfts::{CollectionRole, CollectionRoles};

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

			ItemBalances::<T, I>::insert((collection_id, &who, item_id), amount);

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

			let balance = ItemBalances::<T, I>::get((collection_id, &who, item_id));
			ItemBalances::<T, I>::insert((collection_id, &who, item_id), balance + amount);

			Self::deposit_event(Event::<T, I>::ItemCreated {
				collection_id,
				item_id,
				amount: balance + amount,
			});
		} else {
			return Err(Error::<T, I>::UnknownCollection.into())
		}

		Ok(())
	}
}
