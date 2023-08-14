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
		maybe_supply: Option<u32>,
	) -> DispatchResult {
		if let Some(collection_owner) = T::Nfts::collection_owner(collection) {
			ensure!(
				T::Nfts::is_admin(collection, who) | T::Nfts::is_issuer(collection, who),
				Error::<T, I>::NoPermission
			);

			T::Nfts::mint_into(
				&collection,
				&item,
				&collection_owner,
				&ItemConfig::default(),
				false,
			)?;

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

	/// Adds a specified amount of an item to a collection's finite supply and the balance of
	/// `collection` owner, subject to permissions.
	///
	/// # Parameters
	///
	/// - `who`: The account identifier of the caller attempting to add supply.
	/// - `collection`: The identifier of the collection to which the item belongs.
	/// - `item`: The identifier of the item to add supply for.
	/// - `amount`: The amount to add to both the balance and the finite supply of the item.
	fn do_add_supply(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		// Ensure the caller has the required permission
		if let Some(collection_owner) = T::Nfts::collection_owner(collection) {
			ensure!(
				T::Nfts::is_admin(collection, who) || T::Nfts::is_issuer(collection, who),
				Error::<T, I>::NoPermission
			);

			// Ensure the item's supply is not infinite
			ensure!(
				!Self::is_infinite(collection, item),
				Error::<T, I>::InfiniteSupply
			);

			// Add the item to the collection owner's balance
			Self::add_item_balance(&collection_owner, collection, item, amount)?;

			// Increase the finite supply of the item
			Self::increase_finite_item_supply(collection, item, amount);

			// Emit an event to indicate the successful addition of supply
			Self::deposit_event(Event::<T, I>::ItemAdded {
				who: who.clone(),
				collection: *collection,
				item: *item,
				amount,
			});

			Ok(())
		} else {
			Err(Error::<T, I>::UnknownCollection.into())
		}
	}
}
