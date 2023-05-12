use crate::{types::Item, *};
use frame_support::pallet_prelude::*;
use gafi_support::game::{Amount, Mutable};

impl<T: Config<I>, I: 'static> Mutable<T::AccountId, T::GameId, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{
	fn do_mint(
		who: T::AccountId,
		collection_id: T::CollectionId,
		target: T::AccountId,
	) -> DispatchResult {

		// make a deposit


		// random mint

		Ok(())
	}

	fn do_burn(
		who: T::AccountId,
		collection_id: T::CollectionId,
		item_id: T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		Ok(())
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
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

	fn random_number(total: u32) -> u32 {
		let mut random_number = Self::generate_random_number(1);
		for i in 1..10 {
			if random_number < u32::MAX - u32::MAX % total {
				break
			}

			random_number = Self::generate_random_number(i);
		}
		return random_number
	}

	pub fn random_item(
		source: &Vec<Item<T::ItemId>>,
	) -> Result<(Vec<Item<T::ItemId>>, T::ItemId), Error<T, I>> {
		let mut total_item = 0_u32;
		for item in source.clone() {
			total_item += item.amount;
		}

		ensure!(total_item > 0, Error::<T, I>::SoldOut);

		let position = Self::random_number(total_item);

		let mut new_source = source.clone();
		let rand_item: T::ItemId;

		let mut tmp = 0_u32;
		for i in 0..new_source.len() {
			if new_source[i].amount > 0 && new_source[i].amount + tmp >= position {
				new_source[i] = Item {
					amount: new_source[i].amount - 1,
					item: new_source[i].item,
				};
				rand_item = new_source[i].item;
				return Ok((new_source, rand_item))
			} else {
				tmp += new_source[i].amount;
			}
		}
		Err(Error::<T, I>::SoldOut)
	}
}
