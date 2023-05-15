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
		amount: Amount,
	) -> DispatchResult {
		let mut fee = BalanceOf::<T, I>::default();

		// if collection owner not found, transfer to signer
		let mut collection_owner = who.clone();

		if let Some(owner) = T::Nfts::collection_owner(&collection_id) {
			collection_owner = owner;
		}
		if let Some(config) = GameCollectionConfigOf::<T, I>::get(collection_id) {
			fee = config.mint_settings.mint_settings.price.unwrap_or_default()
		}
		// make a deposit
		<T as pallet::Config<I>>::Currency::transfer(
			&who,
			&collection_owner,
			fee * amount.into(),
			ExistenceRequirement::KeepAlive,
		)?;

		// random mint
		let mut total_item = 0_u32;
		{
			let source = ItemReserve::<T, I>::get(collection_id);

			for item in source.clone() {
				total_item += item.amount;
			}
		}

		ensure!(total_item > 0, Error::<T, I>::SoldOut);
		ensure!(amount <= total_item, Error::<T, I>::ExceedTotalAmount);
		if let Some(max_mint) = Self::get_max_mint_amount(collection_id) {
			ensure!(amount <= max_mint, Error::<T, I>::ExceedAllowedAmount);
		}

		let mut minted_items: Vec<T::ItemId> = [].to_vec();
		for _ in 0..amount {
			let position = Self::random_number(total_item) % total_item;
			match Self::withdraw_reserve(collection_id, position) {
				Ok(item) => {
					Self::add_item_balance(target.clone(), collection_id, item)?;
					minted_items.push(item);
				},
				Err(err) => {
					return Err(err.into())
				},
			};
		}
		Self::deposit_event(Event::<T, I>::Minted {
			minter: who,
			target,
			collection_id,
			minted_items: minted_items,
		});
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

	pub fn withdraw_reserve(
		collection_id: T::CollectionId,
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

	pub fn add_item_balance(
		who: T::AccountId,
		collection: T::CollectionId,
		item: T::ItemId,
	) -> Result<(), Error<T, I>> {
		let amount = ItemBalances::<T, I>::get((&collection, &who, &item));
		ItemBalances::<T, I>::insert((collection, who, item), amount + 1);
		Ok(())
	}

	pub fn get_max_mint_amount(collection: T::CollectionId) -> Option<u32> {
		match GameCollectionConfigOf::<T, I>::get(collection) {
			Some(config) => return config.mint_settings.amount,
			None => None,
		}
	}
}
