use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, Bundle, Package, Trade};

impl<T: Config<I>, I: 'static>
	Trade<T::AccountId, T::CollectionId, T::ItemId, T::TradeId, BalanceOf<T, I>> for Pallet<T, I>
{
	fn do_set_price(
		id: &T::TradeId,
		who: &T::AccountId,
		package: Package<T::CollectionId, T::ItemId>,
		price: BalanceOf<T, I>,
	) -> DispatchResult {
		// ensure available id
		ensure!(
			!BundleOf::<T, I>::contains_key(id),
			Error::<T, I>::TradeIdInUse
		);

		// ensure transferable
		ensure!(
			T::Nfts::can_transfer(&package.collection, &package.item),
			Error::<T, I>::ItemLocked
		);

		// ensure reserve deposit
		<T as Config<I>>::Currency::reserve(&who, T::SaleDeposit::get())?;

		// lock sale items
		Self::lock_item(who, &package.collection, &package.item, package.amount)?;

		PackageOf::<T, I>::insert(id, &package);
		TradeConfigOf::<T, I>::insert(
			id,
			TradeConfig {
				trade: TradeType::Normal,
				owner: who.clone(),
				maybe_price: Some(price),
				maybe_required: None,
			},
		);

		Self::deposit_event(Event::<T, I>::PriceSet {
			id: *id,
			who: who.clone(),
			collection: package.collection,
			item: package.item,
			amount: package.amount,
			price,
		});

		Ok(())
	}

	fn do_buy_item(
		id: &T::TradeId,
		who: &T::AccountId,
		amount: Amount,
		bid_price: BalanceOf<T, I>,
	) -> DispatchResult {
		if let Some(package) = PackageOf::<T, I>::get(id) {
			if let Some(config) = TradeConfigOf::<T, I>::get(id) {
				ensure!(
					config.trade == TradeType::Normal,
					Error::<T, I>::UnknownTrade
				);

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

				// subtract the amount sold
				PackageOf::<T, I>::try_mutate(id, |package| -> DispatchResult {
					match package {
						Some(pack) => {
							pack.amount -= amount;
							Ok(())
						},
						None => Err(Error::<T, I>::InsufficientItemBalance.into()),
					}
				})?;

				Self::deposit_event(Event::<T, I>::ItemBought {
					id: *id,
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

		return Err(Error::<T, I>::NotForSale.into())
	}

	fn do_set_bundle(
		id: &T::TradeId,
		who: &T::AccountId,
		bundle: Bundle<T::CollectionId, T::ItemId>,
		price: BalanceOf<T, I>,
	) -> DispatchResult {
		// ensure available id
		ensure!(
			!BundleOf::<T, I>::contains_key(id),
			Error::<T, I>::TradeIdInUse,
		);

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

		TradeConfigOf::<T, I>::insert(
			id,
			TradeConfig {
				trade: TradeType::Bundle,
				owner: who.clone(),
				maybe_price: Some(price),
				maybe_required: None,
			},
		);

		Self::deposit_event(Event::<T, I>::BundleSet {
			id: *id,
			who: who.clone(),
			price,
		});

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

		if let Some(config) = TradeConfigOf::<T, I>::get(bundle_id) {
			ensure!(
				config.trade == TradeType::Bundle,
				Error::<T, I>::UnknownTrade
			);

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
			<T as pallet::Config<I>>::Currency::unreserve(&config.owner, T::BundleDeposit::get());

			Self::deposit_event(Event::<T, I>::BundleBought {
				id: *bundle_id,
				seller: config.owner,
				buyer: who.clone(),
				price: price,
			});

			return Ok(())
		}
		return Err(Error::<T, I>::NotForSale.into())
	}

	fn do_cancel_price(id: &T::TradeId, who: &T::AccountId) -> DispatchResult {
		if let Some(package) = PackageOf::<T, I>::get(id) {
			if let Some(config) = TradeConfigOf::<T, I>::get(id) {
				// ensure owner
				ensure!(who.eq(&config.owner), Error::<T, I>::NoPermission);

				// unlock items
				Self::unlock_item(who, &package.collection, &package.item, package.amount)?;

				// unreserve
				<T as pallet::Config<I>>::Currency::unreserve(&config.owner, T::SaleDeposit::get());

				// remove storage
				PackageOf::<T, I>::remove(id);
				TradeConfigOf::<T, I>::remove(id);

				Self::deposit_event(Event::<T, I>::TradeCanceled {
					id: *id,
					who: who.clone(),
				});

				return Ok(())
			}
		}
		Err(Error::<T, I>::UnknownTrade.into())
	}

	fn do_cancel_bundle(id: &T::TradeId, who: &T::AccountId) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(id) {
			// ensure owner
			ensure!(who.eq(&config.owner), Error::<T, I>::NoPermission);

			let bundle = BundleOf::<T, I>::get(id);
			// unlock items
			for package in bundle.clone() {
				Self::unlock_item(who, &package.collection, &package.item, package.amount)?;
			}

			// unreserve
			<T as pallet::Config<I>>::Currency::unreserve(&config.owner, T::BundleDeposit::get());

			// remove storage
			BundleOf::<T, I>::remove(id);
			TradeConfigOf::<T, I>::remove(id);

			Self::deposit_event(Event::<T, I>::TradeCanceled {
				id: *id,
				who: who.clone(),
			});
			return Ok(())
		}
		Err(Error::<T, I>::UnknownTrade.into())
	}
}
