/// Item module provides utility functions for pallet-game
use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::game::Bundle;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Generate a random number from a given seed.
	/// Note that there is potential bias introduced by using modulus operator.
	/// You should call this function with different seed values until the random
	/// number lies within `u32::MAX - u32::MAX % n`.
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

	/// Withdraw an item in reserve which item depend on position.
	/// The position of item withdrawal in a sum up from left to right
	/// Example array [(`item`: `amount`)]: [(1, 5), (2, 4), (3, 3)],
	/// With position = 4, result item = 1.
	/// With position = 7, result item = 2.
	/// With position = 10, result item = 3.
	pub(crate) fn withdraw_reserve(
		pool: &T::PoolId,
		position: u32,
	) -> Result<Package<T::CollectionId, T::ItemId>, Error<T, I>> {
		ReserveOf::<T, I>::try_mutate(pool, |reserve_vec| {
			let mut tmp = 0_u32;
			for reserve in reserve_vec.into_iter() {
				if reserve.amount > 0 && reserve.amount + tmp >= position {
					*reserve = reserve.clone().sub(1);
					return Ok(Package {
						collection: reserve.clone().collection,
						item: reserve.clone().item,
						amount: 1,
					})
				} else {
					tmp += reserve.amount;
				}
			}
			return Err(Error::<T, I>::WithdrawReserveFailed.into())
		})
	}

	pub(crate) fn add_total_reserve(pool: &T::PoolId, amount: u32) -> Result<(), Error<T, I>> {
		ensure!(amount > 0, Error::<T, I>::InvalidAmount);
		let total = TotalReserveOf::<T, I>::get(pool);
		TotalReserveOf::<T, I>::insert(pool, total.saturating_add(amount));
		Ok(())
	}

	pub(crate) fn sub_total_reserve(pool: &T::PoolId, amount: u32) -> Result<(), Error<T, I>> {
		ensure!(amount > 0, Error::<T, I>::InvalidAmount);
		let total = TotalReserveOf::<T, I>::get(pool);
		ensure!(total >= amount, Error::<T, I>::SoldOut);
		TotalReserveOf::<T, I>::insert(pool, total.saturating_sub(amount));
		Ok(())
	}

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

	pub(crate) fn move_item(
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

	pub(crate) fn reserved_bundle(
		who: &T::AccountId,
		bundle: Bundle<T::CollectionId, T::ItemId>,
	) -> Result<(), Error<T, I>> {
		for package in bundle {
			Self::reserved_item(who, &package.collection, &package.item, package.amount)?;
		}
		Ok(())
	}

	pub fn count_amount(bundle: &Bundle<T::CollectionId, T::ItemId>) -> u32 {
		let mut counter = 0;
		for package in bundle {
			counter += package.amount;
		}
		counter
	}

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

	pub(crate) fn get_trade_id() -> T::TradeId {
		let id = NextTradeId::<T, I>::get().unwrap_or(T::TradeId::initial_value());
		NextTradeId::<T, I>::set(Some(id.increment()));
		id
	}

	pub(crate) fn get_pool_id() -> T::PoolId {
		let id = NextPoolId::<T, I>::get().unwrap_or(T::PoolId::initial_value());
		NextPoolId::<T, I>::set(Some(id.increment()));
		id
	}
}

// #[cfg(test)]
// #[test]
// fn withdraw_reserve_should_works() {
//     use crate::mock::{new_test_ext, run_to_block, Test, PalletGame};

// 	new_test_ext().execute_with(|| {
// 		run_to_block(2);

// 		let _ = ReserveOf::<Test>::try_mutate(0, |reserve_vec| {
// 			let _ = reserve_vec.try_push(Item::new(1, 9));
// 			let _ = reserve_vec.try_push(Item::new(2, 5));
// 			let _ = reserve_vec.try_push(Item::new(3, 1));
// 			Ok(())
// 		})
// 		.map_err(|_err: Error<Test>| <Error<Test>>::ExceedMaxItem);

// 		let item = PalletGame::withdraw_reserve(&0, 0);
// 		assert_eq!(item.unwrap(), 1);

// 		let item = PalletGame::withdraw_reserve(&0, 9);
// 		assert_eq!(item.unwrap(), 2);

// 		let item = PalletGame::withdraw_reserve(&0, 13);
// 		assert_eq!(item.unwrap(), 3);
// 	})
// }
