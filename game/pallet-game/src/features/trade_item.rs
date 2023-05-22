use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, Bundle, Package, Trade};

impl<T: Config<I>, I: 'static>
	Trade<T::AccountId, T::CollectionId, T::ItemId, T::TradeId, BalanceOf<T, I>> for Pallet<T, I>
{
	fn do_set_price(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		config: &TradeConfig<BalanceOf<T, I>>,
	) -> DispatchResult {
		// ensure balance
		ensure!(
			ItemBalanceOf::<T, I>::get((who, collection, item)) >= config.amount,
			Error::<T, I>::InsufficientItemBalance
		);

		// ensure transferable
		ensure!(
			T::Nfts::can_transfer(collection, item),
			Error::<T, I>::ItemLocked
		);

		// ensure reserve deposit
		<T as Config<I>>::Currency::reserve(&who, T::SaleDeposit::get())?;

		// lock sale items
		Self::lock_item(who, collection, item, config.amount)?;

		TradeConfigOf::<T, I>::insert((who, collection, item), config);

		Self::deposit_event(Event::<T, I>::PriceSet {
			who: who.clone(),
			collection: *collection,
			item: *item,
			amount: config.amount,
			price: config.price,
		});

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
			Self::transfer_lock_item(seller, collection, item, who, amount)?;
			Self::unlock_item(who, collection, item, amount)?;

			{
				let mut new_trade = trade.clone();
				new_trade.amount -= amount;
				TradeConfigOf::<T, I>::insert((seller, collection, item), new_trade);
			}

			Self::deposit_event(Event::<T, I>::ItemBought {
				seller: seller.clone(),
				buyer: who.clone(),
				collection: *collection,
				item: *item,
				amount,
				price: trade.price,
			})
		} else {
			return Err(Error::<T, I>::NotForSale.into())
		}

		Ok(())
	}

	fn do_set_bundle(
		who: &T::AccountId,
		bundle: Bundle<T::CollectionId, T::ItemId>,
		price: BalanceOf<T, I>,
	) -> DispatchResult {
		// ensure ownership
		for package in bundle.clone() {
			ensure!(
				ItemBalanceOf::<T, I>::get((who, package.collection, package.item)) >=
					package.amount,
				Error::<T, I>::InsufficientItemBalance,
			);
		}

		// ensure available id
		let bundle_id = Self::get_bundle_id(&bundle, &who);
		ensure!(
			!BundleOf::<T, I>::contains_key(bundle_id),
			Error::<T, I>::IdExists,
		);

		// lock bundle
		for package in bundle.clone() {
			Self::lock_item(who, &package.collection, &package.item, package.amount)?;
		}

		let _ = BundleOf::<T, I>::try_mutate(bundle_id, |package_vec| {
			package_vec.try_extend(bundle.into_iter())
		})
		.map_err(|_| <Error<T, T>>::ExceedMaxBundle);

		BundleConfigOf::<T, I>::insert(
			bundle_id,
			BundleConfig {
				owner: who.clone(),
				price,
			},
		);

		Ok(())
	}

	fn do_buy_bundle(
		who: &T::AccountId,
		bundle_id: T::TradeId,
		bid_price: BalanceOf<T, I>,
	) -> DispatchResult {
		todo!()
	}
}
