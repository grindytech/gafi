use crate::{types::Item, *};
use frame_support::pallet_prelude::*;
use gafi_support::game::{Amount, CreateItem};
use pallet_nfts::{CollectionRole, CollectionRoles};
impl<T: Config<I>, I: 'static> CreateItem<T::AccountId, T::CollectionId, T::ItemId, ItemConfig>
	for Pallet<T, I>
{
	fn do_create_item(
		who: &T::AccountId,
		collection_id: &T::CollectionId,
		item_id: &T::ItemId,
		config: &ItemConfig,
		amount: Amount,
	) -> DispatchResult {
		// ensure permission
		if let Some(game_id) = GameOf::<T, I>::get(collection_id) {
			ensure!(
				GameRoleOf::<T, I>::get(game_id, &who) ==
					Some(CollectionRoles(
						CollectionRole::Issuer | CollectionRole::Freezer | CollectionRole::Admin
					)),
				Error::<T, I>::NoPermission
			);

			T::Nfts::mint_into(&collection_id, &item_id, &who, &config, false)?;

			// issues new amount of item
			{
				ItemReserve::<T, I>::try_mutate(&collection_id, |reserve_vec| {
					reserve_vec.try_push(Item::new(item_id.clone(), amount))
				})
				.map_err(|_| <Error<T, I>>::ExceedMaxItem)?;

				Self::add_total_reserve(collection_id, amount)?;
			}

			Self::deposit_event(Event::<T, I>::ItemCreated {
				collection_id: *collection_id,
				item_id: *item_id,
				amount,
			});
			Ok(())
		} else {
			return Err(Error::<T, I>::UnknownCollection.into())
		}
	}

	fn do_add_item(
		who: &T::AccountId,
		collection_id: &T::CollectionId,
		item_id: &T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		// ensure permission
		if let Some(game_id) = GameOf::<T, I>::get(collection_id) {
			ensure!(
				GameRoleOf::<T, I>::get(game_id, &who) ==
					Some(CollectionRoles(
						CollectionRole::Issuer | CollectionRole::Freezer | CollectionRole::Admin
					)),
				Error::<T, I>::NoPermission
			);

			// issues amount of item
			{
				ItemReserve::<T, I>::try_mutate(&collection_id, |reserve_vec| {
					let balances = reserve_vec.into_mut();
					for balance in balances {
						if balance.item == *item_id {
							balance.amount += amount;
							return Ok(balance.amount)
						}
					}
					return Err(Error::<T, I>::UnknownItem)
				})
				.map_err(|err| err)?;

				Self::add_total_reserve(collection_id, amount)?;
			}

			Self::deposit_event(Event::<T, I>::ItemCreated {
				collection_id: *collection_id,
				item_id: *item_id,
				amount,
			});
		} else {
			return Err(Error::<T, I>::UnknownCollection.into())
		}

		Ok(())
	}
}
