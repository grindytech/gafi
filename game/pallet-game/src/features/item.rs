use crate::{types::Item, *};
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, Mutable};

impl<T: Config<I>, I: 'static> Mutable<T::AccountId, T::GameId, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{
	fn do_mint(
		who: T::AccountId,
		collection_id: T::CollectionId,
		target: T::AccountId,
	) -> DispatchResult {

		let mut fee = BalanceOf::<T, I>::default();
		
		// if collection owner not found, transfer to signer
		let mut collection_owner = who.clone();

		if let Some(owner) = T::Nfts::collection_owner(&collection_id) {
			collection_owner = owner;
		}
		if let Some(config) = GameCollectionConfigOf::<T, I>::get(collection_id) {
			fee = config.mint_settings.mint_settings.price.unwrap();
		}
		// make a deposit
		<T as pallet::Config<I>>::Currency::transfer(
			&who,
			&collection_owner,
			fee,
			ExistenceRequirement::KeepAlive,
		)?;

		// random mint
		if let Ok(item) = Self::random_item(collection_id) {
			Self::add_item_balance(target, collection_id, item)?;
		}

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
		collection_id: T::CollectionId,
	) -> Result<T::ItemId, Error<T, I>> {

		let mut source = ItemReserve::<T, I>::get(collection_id);

		let mut total_item = 0_u32;
		for item in source.clone() {
			total_item += item.amount;
		}

		ensure!(total_item > 0, Error::<T, I>::SoldOut);

		let position = Self::random_number(total_item);

		let rand_item: T::ItemId;

		let mut tmp = 0_u32;
		for i in 0..source.len() {
			if source[i].amount > 0 && source[i].amount + tmp >= position {
				source[i] = Item {
					amount: source[i].amount - 1,
					item: source[i].item,
				};
				rand_item = source[i].item;
				return Ok(rand_item);
			} else {
				tmp += source[i].amount;
			}
		}
		Err(Error::<T, I>::SoldOut)
	}

	pub fn add_item_balance(
		who: T::AccountId,
		collection: T::CollectionId,
		item: T::ItemId,
	) -> Result<(), Error<T, I>> {
		let amount = ItemBalances::<T, I>::get((&collection, &who, &item));
		ItemBalances::<T, I>::insert((collection, who, item), amount + 1);
		Ok(())
	}
}
