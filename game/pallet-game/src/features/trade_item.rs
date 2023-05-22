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
		id: &T::TradeId,
		who: &T::AccountId,
		bundle: Bundle<T::CollectionId, T::ItemId>,
		price: BalanceOf<T, I>,
	) -> DispatchResult {
		// ensure available id
		ensure!(!BundleOf::<T, I>::contains_key(id), Error::<T, I>::IdExists,);

		// ensure ownership
		for package in bundle.clone() {
			ensure!(
				ItemBalanceOf::<T, I>::get((who, package.collection, package.item)) >=
					package.amount,
				Error::<T, I>::InsufficientItemBalance,
			);
		}

		<T as Config<I>>::Currency::reserve(&who, T::BundleDeposit::get())?;

		// lock bundle
		for package in bundle.clone() {
			Self::lock_item(who, &package.collection, &package.item, package.amount)?;
		}

		<BundleOf<T, I>>::try_mutate(id, |package_vec| -> DispatchResult {
			package_vec
				.try_append(bundle.clone().into_mut())
				.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
			Ok(())
		})?;

		NextTradeId::<T, I>::set(Some(id.increment()));

		BundleConfigOf::<T, I>::insert(
			id,
			BundleConfig {
				owner: who.clone(),
				price,
			},
		);

		Ok(())
	}

	fn do_buy_bundle(
		bundle_id: &T::TradeId,
		who: &T::AccountId,
		bid_price: BalanceOf<T, I>,
	) -> DispatchResult {
		let bundle = BundleOf::<T, I>::get(bundle_id);

		// ensure item can be transfer
		for pack in bundle.clone() {
			ensure!(
				T::Nfts::can_transfer(&pack.collection, &pack.item),
				Error::<T, I>::ItemLocked
			);
		}

		if let Some(bundle_config) = BundleConfigOf::<T, I>::get(bundle_id) {
			// check price
			ensure!(bid_price >= bundle_config.price, Error::<T, I>::BidTooLow);

			// make deposit
			<T as pallet::Config<I>>::Currency::transfer(
				&who,
				&bundle_config.owner,
				bundle_config.price,
				ExistenceRequirement::KeepAlive,
			)?;

			// transfer items
			for package in bundle.clone() {
				Self::transfer_lock_item(
					&bundle_config.owner,
					&package.collection,
					&package.item,
					who,
					package.amount,
				)?;

				Self::unlock_item(
					who,
					&package.collection,
					&package.item,
					package.amount,
				)?;
			}
			<T as pallet::Config<I>>::Currency::unreserve(&bundle_config.owner, T::BundleDeposit::get());
		}

		Ok(())
	}
}
