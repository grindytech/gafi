/// Item module provides utility functions for pallet-game
use crate::*;
use frame_support::pallet_prelude::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Generate a random number from a given seed.
	/// Note that there is potential bias introduced by using modulus operator.
	/// You should call this function with different seed values until the random
	/// number lies within `u32::MAX - u32::MAX % n`.
	// SBP-M2: Please resolve this TODO
	/// TODO: deal with randomness freshness
	/// https://github.com/grindytech/substrate/issues/8311
	fn generate_random_number(seed: u32) -> u32 {
		let (random_seed, _) = T::Randomness::random(&(T::PalletId::get(), seed).encode());
		let random_number = <u32>::decode(&mut random_seed.as_ref())
			.expect("secure hashes should always be bigger than u32; qed");
		random_number
	}

	/// Generate a random number from a given seed.
	/// Generated number lies with `0 - total`.
	pub(crate) fn random_number(total: u32, seed: u32) -> Option<u32> {
		if total == 0 {
			return None
		}

		let mut random_number = Self::generate_random_number(seed);
		for _ in 1..10 {
			if random_number < u32::MAX.saturating_sub(u32::MAX % total) {
				break
			}

			random_number = Self::generate_random_number(seed);
		}

		Some(random_number % total)
	}

	/// Generate a random number from the off-chain worker's random seed
	pub(crate) fn gen_random() -> u32 {
		let seed = RandomSeed::<T, I>::get();

		let random = <u32>::decode(&mut TrailingZeroInput::new(seed.as_ref()))
			.expect("input is padded with zeroes; qed");

		random
	}

	/// Transfer an `amount` of `item` in `collection` from `from` to `to`.
	pub(crate) fn transfer_item(
		from: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		to: &T::AccountId,
		amount: u32,
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
		amount: u32,
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
		amount: u32,
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
		amount: u32,
	) -> Result<(), Error<T, I>> {
		ensure!(amount > 0, Error::<T, I>::InvalidAmount);
		let balance = ItemBalanceOf::<T, I>::get((&who, &collection, &item));
		ensure!(balance >= amount, Error::<T, I>::InsufficientItemBalance);
		ItemBalanceOf::<T, I>::insert((who, collection, item), balance.saturating_sub(amount));
		Ok(())
	}

	/// Add a new `amount` of reserved `item` in `collection` to `who`.
	fn add_reserved_balance(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: u32,
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
		amount: u32,
	) -> Result<(), Error<T, I>> {
		ensure!(amount > 0, Error::<T, I>::InvalidAmount);
		let balance = ReservedBalanceOf::<T, I>::get((who, collection, item));
		ensure!(
			balance >= amount,
			Error::<T, I>::InsufficientReservedBalance
		);
		ReservedBalanceOf::<T, I>::insert((who, collection, item), balance.saturating_sub(amount));
		Ok(())
	}

	/// Lock `amount` of `item` in `collection` of `who`.
	pub(crate) fn reserved_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: u32,
	) -> Result<(), Error<T, I>> {
		Self::sub_item_balance(who, collection, item, amount)?;
		Self::add_reserved_balance(who, collection, item, amount)?;
		Ok(())
	}

	/// Calculate the total weight in a `table` of loot.
	pub fn total_weight(table: &LootTable<T::CollectionId, T::ItemId>) -> u32 {
		let mut counter = 0;
		for package in table {
			counter += package.weight;
		}
		counter
	}

	/// Unlock `amount` of `item` in `collection` of `who`.
	pub(crate) fn unreserved_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: u32,
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
		amount: u32,
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
		// SBP-M2: Can be simplified like this:
		// `SupplyOf::<T, I>::get(collection, item).map(|maybe_supply| maybe_supply.is_none()).unwrap_or_default()`
		match SupplyOf::<T, I>::get(collection, item) {
			Some(maybe_supply) => match maybe_supply {
				Some(_) => return false,
				None => return true,
			},
			None => false,
		}
	}
}
