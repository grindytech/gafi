/// Item module provides utility functions for pallet-game
use crate::*;
use frame_support::pallet_prelude::*;
use sp_runtime::traits::Saturating;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Transfer an `amount` of `item` in `collection` from `from` to `to`.
	pub(crate) fn transfer_item(
		from: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		to: &T::AccountId,
		amount: Amount,
	) -> Result<(), Error<T, I>> {
		Self::sub_item_balance(from, collection, item, amount)?;
		Self::add_item_balance(to, collection, item, amount)?;
		Ok(())
	}

	/// Convert an `amount` of `old_item` to an `amount` of `new_item` in the `collection` of `who`.
	pub(crate) fn convert_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		old_item: &T::ItemId,
		new_item: &T::ItemId,
		amount: Amount,
	) -> Result<(), Error<T, I>> {
		Self::sub_item_balance(who, collection, old_item, amount)?;
		Self::add_item_balance(who, collection, new_item, amount)?;
		Ok(())
	}

	/// Add a new `amount` of `item` in `collection` to `who`.
	pub(crate) fn add_item_balance(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> Result<(), Error<T, I>> {
		ensure!(amount > 0, Error::<T, I>::InvalidAmount);
		let balance = ItemBalanceOf::<T, I>::get((&who, &collection, &item));
		ItemBalanceOf::<T, I>::insert((who, collection, item), balance.saturating_add(amount));
		Ok(())
	}

	/// Subtract a new `amount` of `item` in `collection` to `who`.
	pub(crate) fn sub_item_balance(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> Result<(), Error<T, I>> {
		ensure!(amount > 0, Error::<T, I>::InvalidAmount);
		let balance = ItemBalanceOf::<T, I>::get((&who, &collection, &item));
		ensure!(balance >= amount, Error::<T, I>::InsufficientItemBalance);

		let new_balance = balance.saturating_sub(amount);
		if new_balance == 0 {
			ItemBalanceOf::<T, I>::remove((who, collection, item));
		} else {
			ItemBalanceOf::<T, I>::insert((who, collection, item), new_balance);
		}
		Ok(())
	}

	/// Add a new `amount` of reserved `item` in `collection` to `who`.
	fn add_reserved_balance(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> Result<(), Error<T, I>> {
		ensure!(amount > 0, Error::<T, I>::InvalidAmount);
		let balance = ReservedBalanceOf::<T, I>::get((&who, &collection, &item));
		ReservedBalanceOf::<T, I>::insert((who, collection, item), balance.saturating_add(amount));
		Ok(())
	}

	/// Subtract a new `amount` of reserved `item` in `collection` to `who`.
	fn sub_reserved_balance(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> Result<(), Error<T, I>> {
		ensure!(amount > 0, Error::<T, I>::InvalidAmount);
		let balance = ReservedBalanceOf::<T, I>::get((who, collection, item));
		ensure!(
			balance >= amount,
			Error::<T, I>::InsufficientReservedBalance
		);
		let new_balance = balance.saturating_sub(amount);
		if new_balance == 0 {
			ReservedBalanceOf::<T, I>::remove((who, collection, item));
		} else {
			ReservedBalanceOf::<T, I>::insert((who, collection, item), new_balance);
		}
		Ok(())
	}

	/// Lock `amount` of `item` in `collection` of `who`.
	pub(crate) fn reserved_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> Result<(), Error<T, I>> {
		Self::sub_item_balance(who, collection, item, amount)?;
		Self::add_reserved_balance(who, collection, item, amount)?;
		Ok(())
	}

	/// Calculate the total weight in a `table` of loot.
	pub fn total_weight(table: &LootTable<T::CollectionId, T::ItemId>) -> u32 {
		let mut counter = 0;
		for package in table {
			counter = counter.saturating_add(package.weight);
		}
		counter
	}

	/// Unlock `amount` of `item` in `collection` of `who`.
	pub(crate) fn unreserved_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> Result<(), Error<T, I>> {
		Self::sub_reserved_balance(who, collection, item, amount)?;
		Self::add_item_balance(who, collection, item, amount)?;
		Ok(())
	}

	///  Move the item reserved item balance of one account into the item balance of another,
	/// according to `status`.
	pub(crate) fn repatriate_reserved_item(
		slashed: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		beneficiary: &T::AccountId,
		amount: Amount,
		status: ItemBalanceStatus,
	) -> Result<(), Error<T, I>> {
		Self::sub_reserved_balance(slashed, collection, item, amount)?;
		match status {
			ItemBalanceStatus::Reserved => {
				Self::add_reserved_balance(beneficiary, collection, item, amount)?;
			},
			ItemBalanceStatus::Free => {
				Self::add_item_balance(beneficiary, collection, item, amount)?;
			},
		};
		Ok(())
	}

	/// Get the available game id and increase the id by 1.
	pub(crate) fn get_game_id() -> T::GameId {
		let id = NextGameId::<T, I>::get().unwrap_or(T::GameId::initial_value());
		NextGameId::<T, I>::set(Some(id.increment()));
		id
	}

	/// Get the available trade id and increase the id by 1.
	pub(crate) fn get_trade_id() -> T::TradeId {
		let id = NextTradeId::<T, I>::get().unwrap_or(T::TradeId::initial_value());
		NextTradeId::<T, I>::set(Some(id.increment()));
		id
	}

	/// Get the available pool id and increase the id by 1.
	pub(crate) fn get_pool_id() -> T::PoolId {
		let id = NextPoolId::<T, I>::get().unwrap_or(T::PoolId::initial_value());
		NextPoolId::<T, I>::set(Some(id.increment()));
		id
	}

	/// Check if `item` in `collection` is in infinite supply.
	pub(crate) fn is_infinite(collection: &T::CollectionId, item: &T::ItemId) -> bool {
		SupplyOf::<T, I>::get(collection, item)
			.map(|maybe_supply| maybe_supply.is_none())
			.unwrap_or_default()
	}

	/// Decreases the finite supply of a specific item within a collection by a given amount.
	///
	/// If the collection and item exist, their finite supply is decremented by the specified
	/// amount, with saturation to prevent underflow.
	///
	/// # Parameters
	///
	/// - `collection`: The identifier of the collection.
	/// - `item`: The identifier of the item.
	/// - `amount`: The amount to decrease the item's supply by.
	pub(crate) fn decrease_finite_item_supply(
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) {
		if let Some(Some(existing_supply)) = SupplyOf::<T, I>::get(collection, item) {
			SupplyOf::<T, I>::insert(
				collection,
				item,
				Some(existing_supply.saturating_sub(amount)),
			);
		}
	}

	/// Increases the finite supply of a specific item within a collection by a given amount.
	///
	/// If the collection and item exist, their finite supply is incremented by the specified
	/// amount, with saturation to prevent overflow.
	///
	/// # Parameters
	///
	/// - `collection`: The identifier of the collection.
	/// - `item`: The identifier of the item.
	/// - `amount`: The amount to increase the item's supply by.
	pub(crate) fn increase_finite_item_supply(
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) {
		if let Some(Some(existing_supply)) = SupplyOf::<T, I>::get(collection, item) {
			SupplyOf::<T, I>::insert(
				collection,
				item,
				Some(existing_supply.saturating_add(amount)),
			);
		}
	}
}
