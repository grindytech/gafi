use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, MutateItem};

impl<T: Config<I>, I: 'static> MutateItem<T::AccountId, T::GameId, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{
	fn do_mint(
		who: &T::AccountId,
		collection_id: &T::CollectionId,
		target: &T::AccountId,
		amount: Amount,
	) -> DispatchResult {
		// validating item amount
		{
			let total_item = TotalReserveOf::<T, I>::get(collection_id);

			ensure!(total_item > 0, Error::<T, I>::SoldOut);
			ensure!(amount <= total_item, Error::<T, I>::ExceedTotalAmount);

			ensure!(
				amount <= T::MaxMintItem::get(),
				Error::<T, I>::ExceedAllowedAmount
			);
		}

		// deposit mining fee
		// if collection owner not found, skip deposit
		if let Some(owner) = T::Nfts::collection_owner(&collection_id) {
			if let Some(config) = GameCollectionConfigOf::<T, I>::get(collection_id) {
				let fee = config.mint_settings.price.unwrap_or_default();
				// make a deposit
				<T as pallet::Config<I>>::Currency::transfer(
					&who,
					&owner,
					fee * amount.into(),
					ExistenceRequirement::KeepAlive,
				)?;
			}
		}

		// random minting
		let mut minted_items: Vec<T::ItemId> = [].to_vec();
		{
			let mut total_item = TotalReserveOf::<T, I>::get(collection_id);
			let mut maybe_position = Some(Self::gen_random());
			for i in 0..amount {
				if let Some(position) = maybe_position {
					maybe_position = Self::random_number(total_item, position);
					total_item = total_item.saturating_sub(i);

					match Self::withdraw_reserve(collection_id, position) {
						Ok(item) => {
							Self::add_item_balance(&target, &collection_id, &item, 1)?;
							minted_items.push(item);
						},
						Err(err) => return Err(err.into()),
					};
				}
			}
			Self::minus_total_reserve(collection_id, amount)?;
		}

		Self::deposit_event(Event::<T, I>::Minted {
			minter: who.clone(),
			target: target.clone(),
			collection_id: *collection_id,
			minted_items,
		});
		Ok(())
	}

	fn do_burn(
		who: &T::AccountId,
		collection_id: &T::CollectionId,
		item_id: &T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		let item_balance = ItemBalances::<T, I>::get((&who, collection_id, item_id));
		ensure!(
			amount <= item_balance,
			Error::<T, I>::InsufficientItemBalance
		);

		ItemBalances::<T, I>::insert((&who, collection_id, item_id), item_balance - amount);

		Self::deposit_event(Event::<T, I>::Burned {
			collection_id: *collection_id,
			item_id: *item_id,
			amount,
		});
		Ok(())
	}
}
