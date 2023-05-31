use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, MutateItem};

impl<T: Config<I>, I: 'static> MutateItem<T::AccountId, T::GameId, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{
	fn do_mint(
		who: &T::AccountId,
		collection: &T::CollectionId,
		target: &T::AccountId,
		amount: Amount,
	) -> DispatchResult {
		// validating item amount
		{
			let total_item = TotalReserveOf::<T, I>::get(collection);

			ensure!(total_item > 0, Error::<T, I>::SoldOut);
			ensure!(amount <= total_item, Error::<T, I>::ExceedTotalAmount);

			ensure!(
				amount <= T::MaxMintItem::get(),
				Error::<T, I>::ExceedAllowedAmount
			);
		}

		// deposit mining fee
		// if collection owner not found, skip deposit
		if let Some(owner) = T::Nfts::collection_owner(&collection) {
			if let Some(fee) = MintingFeeOf::<T, I>::get(collection) {
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
			let mut total_item = TotalReserveOf::<T, I>::get(collection);
			let mut maybe_position = Self::random_number(total_item, Self::gen_random());
			for _ in 0..amount {
				if let Some(position) = maybe_position {
					match Self::withdraw_reserve(collection, position) {
						Ok(item) => {
							Self::add_item_balance(&target, &collection, &item, 1)?;
							minted_items.push(item);
						},
						Err(err) => return Err(err.into()),
					};
					total_item = total_item.saturating_sub(1);
					maybe_position = Self::random_number(total_item, position);
				} else {
					return Err(Error::<T, I>::SoldOut.into())
				}
			}
			Self::sub_total_reserve(collection, amount)?;
		}

		Self::deposit_event(Event::<T, I>::Minted {
			who: who.clone(),
			target: target.clone(),
			collection: *collection,
			minted_items,
		});
		Ok(())
	}

	fn do_burn(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		Self::sub_item_balance(who, collection, item, amount)?;

		Self::deposit_event(Event::<T, I>::Burned {
			who: who.clone(),
			collection: *collection,
			item: *item,
			amount,
		});
		Ok(())
	}
}
