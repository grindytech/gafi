use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, Trade};

impl<T: Config<I>, I: 'static> Trade<T::AccountId, T::CollectionId, T::ItemId, BalanceOf<T, I>>
	for Pallet<T, I>
{
	fn do_set_price(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		config: &TradeConfig<BalanceOf<T, I>>,
	) -> DispatchResult {
		// ensure balance
		ensure!(
			ItemBalances::<T, I>::get((who, collection, item)) >= config.amount,
			Error::<T, I>::InsufficientItemBalance
		);

		// ensure transferable
		ensure!(
			T::Nfts::can_transfer(collection, item),
			Error::<T, I>::ItemLocked
		);

		TradeConfigOf::<T, I>::insert((who, collection, item), config);

		Ok(())
	}

	fn do_buy_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		seller: &T::AccountId,
		amount: Amount,
		bid_price: BalanceOf<T, I>,
	) -> DispatchResult {
		// ensure item can be transfer
		ensure!(
			T::Nfts::can_transfer(collection, item),
			Error::<T, I>::ItemLocked
		);

		// ensure trade
		if let Some(trade) = TradeConfigOf::<T, I>::get((seller, collection, item)) {
			ensure!(trade.amount > 0, Error::<T, I>::SoldOut);
			
			// sell all case
			if let Some(moq) = trade.min_order_quantity {
				if trade.amount <= moq {
					ensure!(amount == trade.amount, Error::<T, I>::BuyAllOnly);
				} else {
					// check min order quantity
					ensure!(amount >= moq, Error::<T, I>::AmountUnacceptable);
				}
			} else {
				ensure!(amount == trade.amount, Error::<T, I>::BuyAllOnly);
			}

			// check price
			ensure!(bid_price >= trade.price, Error::<T, I>::BidTooLow);

			// make deposit
			<T as pallet::Config<I>>::Currency::transfer(
				&who,
				&seller,
				trade.price * amount.into(),
				ExistenceRequirement::KeepAlive,
			)?;

			// transfer item
			Self::transfer_item(seller, collection, item, who, amount)?;

			{
				let mut new_trade = trade.clone();
				new_trade.amount -= amount;
				TradeConfigOf::<T, I>::insert((seller, collection, item), new_trade);
			}
		} else {
			return Err(Error::<T, I>::NotForSale.into())
		}

		Ok(())
	}
}
