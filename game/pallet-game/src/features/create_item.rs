use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::game::{Amount, CreateItem};
impl<T: Config<I>, I: 'static> CreateItem<T::AccountId, T::CollectionId, T::ItemId, ItemConfig>
	for Pallet<T, I>
{
	fn do_create_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		config: &ItemConfig,
		maybe_supply: Option<u32>,
	) -> DispatchResult {
		if let Some(collection_owner) = T::Nfts::collection_owner(collection) {
			ensure!(
				T::Nfts::is_admin(collection, who) | T::Nfts::is_issuer(collection, who),
				Error::<T, I>::NoPermission
			);

			T::Nfts::mint_into(&collection, &item, &collection_owner, &config, false)?;

			if let Some(supply) = maybe_supply {
				// issues new amount of item
				Self::add_item_balance(&collection_owner, collection, item, supply)?;
			}
			SupplyOf::<T, I>::insert(collection, item, maybe_supply);

			Self::deposit_event(Event::<T, I>::ItemCreated {
				who: who.clone(),
				collection: *collection,
				item: *item,
				maybe_supply,
			});

			return Ok(())
		}
		return Err(Error::<T, I>::UnknownCollection.into())
	}

	fn do_add_supply(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		// ensure permission
		if let Some(collection_owner) = T::Nfts::collection_owner(collection) {
			ensure!(
				T::Nfts::is_admin(collection, who) | T::Nfts::is_issuer(collection, who),
				Error::<T, I>::NoPermission
			);
			let maybe_supply = SupplyOf::<T, I>::get(collection, item);
			if let Some(supply) = maybe_supply {
				match supply {
					Some(val) => {
						let new_supply = val + amount;
						Self::add_item_balance(&collection_owner, collection, item, amount)?;
						SupplyOf::<T, I>::insert(collection, item, Some(new_supply));
					},
					None => return Err(Error::<T, I>::InfiniteSupply.into()),
				};
			} else {
				return Err(Error::<T, I>::UnknownItem.into())
			}

			Self::deposit_event(Event::<T, I>::ItemAdded {
				who: who.clone(),
				collection: *collection,
				item: *item,
				amount,
			});
			return Ok(())
		}
		return Err(Error::<T, I>::UnknownCollection.into())
	}
}
