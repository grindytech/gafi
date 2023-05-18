use crate::{*};
use frame_support::pallet_prelude::*;
use crate::types::Item;

impl<T: Config<I>, I: 'static> Pallet<T, I>
{
    /// Generate a random number from a given seed.
	/// Note that there is potential bias introduced by using modulus operator.
	/// You should call this function with different seed values until the random
	/// number lies within `u32::MAX - u32::MAX % n`.
	/// TODO: deal with randomness freshness
	/// https://github.com/paritytech/substrate/issues/8311
	pub fn generate_random_number(seed: u32) -> u32 {
		let (random_seed, _) = T::Randomness::random(&(T::PalletId::get(), seed).encode());
		let random_number = <u32>::decode(&mut random_seed.as_ref())
			.expect("secure hashes should always be bigger than u32; qed");
		random_number
	}

	pub(crate) fn random_number(total: u32, seed: u32) -> u32 {
		let mut random_number = Self::generate_random_number(seed);
		for _ in 1..10 {
			if random_number < u32::MAX - u32::MAX % total {
				break
			}

			random_number = Self::generate_random_number(seed);
		}

		random_number % total
	}

	pub fn withdraw_reserve(
		collection_id: &T::CollectionId,
		position: u32,
	) -> Result<T::ItemId, Error<T, I>> {
		let result = ItemReserve::<T, I>::try_mutate(collection_id, |reserve_vec| {
			let mut tmp = 0_u32;
			for reserve in reserve_vec.into_iter() {
				if reserve.amount > 0 && reserve.amount + tmp >= position {
					*reserve = Item {
						amount: reserve.amount - 1,
						item: reserve.item,
					};
					return Ok(*reserve)
				} else {
					tmp += reserve.amount;
				}
			}
			Err(Error::<T, I>::WithdrawReserveFailed)
		})
		.map_err(|_| Error::<T, I>::WithdrawReserveFailed);

		match result {
			Ok(item) => Ok(item.item),
			Err(err) => Err(err),
		}
	}

	pub(crate) fn add_total_reserve(
		collection: &T::CollectionId,
		amount: u32,
	) -> Result<(), Error<T, I>> {
		let total = TotalReserveOf::<T, I>::get(collection);
		TotalReserveOf::<T, I>::insert(collection, total + amount);
		Ok(())
	}

	pub fn minus_total_reserve(
		collection: &T::CollectionId,
		amount: u32,
	) -> Result<(), Error<T, I>> {
		let total = TotalReserveOf::<T, I>::get(collection);
		ensure!(total >= amount, Error::<T, I>::SoldOut);
		TotalReserveOf::<T, I>::insert(collection, total - amount);
		Ok(())
	}

	pub fn add_item_balance(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: u32,
	) -> Result<(), Error<T, I>> {
		let balance = ItemBalances::<T, I>::get((&who, &collection, &item));
		ItemBalances::<T, I>::insert((who, collection, item), balance + amount);
		Ok(())
	}

	pub fn minus_item_balance(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: u32,
	) -> Result<(), Error<T, I>> {
		let balance = ItemBalances::<T, I>::get((&who, &collection, &item));
		ensure!(balance >= amount, Error::<T, I>::InsufficientItemBalance);
		ItemBalances::<T, I>::insert((who, collection, item), balance - amount);
		Ok(())
	}
}