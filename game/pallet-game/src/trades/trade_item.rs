use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, Bundle, Package, Retail, TradeType};

impl<T: Config<I>, I: 'static>
	Retail<T::AccountId, T::CollectionId, T::ItemId, T::TradeId, BalanceOf<T, I>> for Pallet<T, I>
{
	fn do_set_price(
		trade: &T::TradeId,
		who: &T::AccountId,
		package: Package<T::CollectionId, T::ItemId>,
		price: BalanceOf<T, I>,
	) -> DispatchResult {
		// ensure available trade
		ensure!(
			!BundleOf::<T, I>::contains_key(trade),
			Error::<T, I>::TradeIdInUse
		);

		// ensure transferable
		ensure!(
			T::Nfts::can_transfer(&package.collection, &package.item),
			Error::<T, I>::ItemLocked
		);

		// ensure reserve deposit
		<T as Config<I>>::Currency::reserve(&who, T::BundleDeposit::get())?;

		// lock sale items
		Self::lock_item(who, &package.collection, &package.item, package.amount)?;

		<BundleOf<T, I>>::try_mutate(trade, |package_vec| -> DispatchResult {
			package_vec
				.try_push(package.clone())
				.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
			Ok(())
		})?;

		TradeConfigOf::<T, I>::insert(
			trade,
			TradeConfig {
				trade: TradeType::Normal,
				owner: who.clone(),
				maybe_price: Some(price),
				maybe_required: None,
			},
		);

		Self::deposit_event(Event::<T, I>::PriceSet {
			trade: *trade,
			who: who.clone(),
			collection: package.collection,
			item: package.item,
			amount: package.amount,
			price,
		});

		Ok(())
	}

	fn do_buy_item(
		trade: &T::TradeId,
		who: &T::AccountId,
		amount: Amount,
		bid_price: BalanceOf<T, I>,
	) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(trade) {
			ensure!(
				config.trade == TradeType::Normal,
				Error::<T, I>::NotSetPrice
			);

			if let Some(package) = BundleOf::<T, I>::get(trade).first() {
				// ensure item can be transfer
				ensure!(
					T::Nfts::can_transfer(&package.collection, &package.item),
					Error::<T, I>::ItemLocked
				);

				// ensure trade
				ensure!(package.amount >= amount, Error::<T, I>::SoldOut);

				// check price
				let price = config.maybe_price.unwrap_or_default();
				ensure!(bid_price >= price, Error::<T, I>::BidTooLow);

				// make deposit
				<T as pallet::Config<I>>::Currency::transfer(
					&who,
					&config.owner,
					price * amount.into(),
					ExistenceRequirement::KeepAlive,
				)?;

				// transfer item
				Self::repatriate_lock_item(
					&config.owner,
					&package.collection,
					&package.item,
					who,
					amount,
					ItemBalanceStatus::Free,
				)?;

				let new_package =
					Package::new(package.collection, package.item, package.amount - amount);

				<BundleOf<T, I>>::try_mutate(trade, |package_vec| -> DispatchResult {
					*package_vec = BundleFor::<T, I>::try_from([new_package].to_vec())
						.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
					Ok(())
				})?;

				Self::deposit_event(Event::<T, I>::ItemBought {
					trade: *trade,
					seller: config.owner,
					buyer: who.clone(),
					collection: package.collection,
					item: package.item,
					amount,
					price,
				});
				return Ok(())
			}
		}

		return Err(Error::<T, I>::UnknownTrade.into())
	}

	fn do_set_bundle(
		trade: &T::TradeId,
		who: &T::AccountId,
		bundle: Bundle<T::CollectionId, T::ItemId>,
		price: BalanceOf<T, I>,
	) -> DispatchResult {
		// ensure available trade
		ensure!(
			!BundleOf::<T, I>::contains_key(trade),
			Error::<T, I>::TradeIdInUse,
		);

		<T as Config<I>>::Currency::reserve(&who, T::BundleDeposit::get())?;

		// lock bundle
		for package in bundle.clone() {
			Self::lock_item(who, &package.collection, &package.item, package.amount)?;
		}

		<BundleOf<T, I>>::try_mutate(trade, |package_vec| -> DispatchResult {
			package_vec
				.try_append(bundle.clone().into_mut())
				.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
			Ok(())
		})?;

		TradeConfigOf::<T, I>::insert(
			trade,
			TradeConfig {
				trade: TradeType::Bundle,
				owner: who.clone(),
				maybe_price: Some(price),
				maybe_required: None,
			},
		);

		Self::deposit_event(Event::<T, I>::BundleSet {
			trade: *trade,
			who: who.clone(),
			bundle,
			price,
		});

		Ok(())
	}

	fn do_buy_bundle(
		trade: &T::TradeId,
		who: &T::AccountId,
		bid_price: BalanceOf<T, I>,
	) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(trade) {
			ensure!(config.trade == TradeType::Bundle, Error::<T, I>::NotBundle);

			let bundle = BundleOf::<T, I>::get(trade);
			// ensure item can be transfer
			for pack in bundle.clone() {
				ensure!(
					T::Nfts::can_transfer(&pack.collection, &pack.item),
					Error::<T, I>::ItemLocked
				);
			}

			let price = config.maybe_price.unwrap_or_default();

			// check price
			ensure!(bid_price >= price, Error::<T, I>::BidTooLow);

			// make deposit
			<T as pallet::Config<I>>::Currency::transfer(
				&who,
				&config.owner,
				price,
				ExistenceRequirement::KeepAlive,
			)?;

			// transfer items
			for package in bundle.clone() {
				Self::repatriate_lock_item(
					&config.owner,
					&package.collection,
					&package.item,
					who,
					package.amount,
					ItemBalanceStatus::Free,
				)?;
			}

			// end trade
			<T as pallet::Config<I>>::Currency::unreserve(&config.owner, T::BundleDeposit::get());
			BundleOf::<T, I>::remove(trade);
			TradeConfigOf::<T, I>::remove(trade);

			Self::deposit_event(Event::<T, I>::BundleBought {
				trade: *trade,
				seller: config.owner,
				buyer: who.clone(),
				price,
			});

			return Ok(())
		}
		return Err(Error::<T, I>::UnknownTrade.into())
	}

	fn do_cancel_price(trade: &T::TradeId, who: &T::AccountId) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(trade) {
			ensure!(
				config.trade == TradeType::Normal,
				Error::<T, I>::NotSetPrice
			);

			if let Some(package) = BundleOf::<T, I>::get(trade).first() {
				// ensure owner
				ensure!(who.eq(&config.owner), Error::<T, I>::NoPermission);

				// unlock items
				Self::unlock_item(who, &package.collection, &package.item, package.amount)?;

				// end trade
				<T as pallet::Config<I>>::Currency::unreserve(
					&config.owner,
					T::BundleDeposit::get(),
				);
				BundleOf::<T, I>::remove(trade);
				TradeConfigOf::<T, I>::remove(trade);

				Self::deposit_event(Event::<T, I>::TradeCanceled {
					trade: *trade,
					who: who.clone(),
				});

				return Ok(())
			}
		}
		Err(Error::<T, I>::UnknownTrade.into())
	}

	fn do_cancel_bundle(trade: &T::TradeId, who: &T::AccountId) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(trade) {
			ensure!(config.trade == TradeType::Bundle, Error::<T, I>::NotBundle);

			// ensure owner
			ensure!(who.eq(&config.owner), Error::<T, I>::NoPermission);

			let bundle = BundleOf::<T, I>::get(trade);
			// unlock items
			for package in bundle.clone() {
				Self::unlock_item(who, &package.collection, &package.item, package.amount)?;
			}

			// end trade
			<T as pallet::Config<I>>::Currency::unreserve(&config.owner, T::BundleDeposit::get());
			BundleOf::<T, I>::remove(trade);
			TradeConfigOf::<T, I>::remove(trade);

			Self::deposit_event(Event::<T, I>::TradeCanceled {
				trade: *trade,
				who: who.clone(),
			});
			return Ok(())
		}
		Err(Error::<T, I>::UnknownTrade.into())
	}

	fn do_cancel_trade(
		trade: &T::TradeId,
		who: &T::AccountId,
		trade_type: TradeType,
	) -> DispatchResult {
		match trade_type {
			TradeType::Normal => {
				Self::do_cancel_price(trade, who)?;
			},
			TradeType::Bundle => {
				Self::do_cancel_bundle(trade, who)?;
			},
			TradeType::Wishlist => {
				Self::do_cancel_wishlist(trade, who)?;
			},
			TradeType::Auction => {
				Self::do_claim_auction(trade)?;
			},
			TradeType::Swap => {
				Self::do_cancel_swap(trade, who)?;
			},
		};
		Err(Error::<T, I>::UnknownTrade.into())
	}
}
