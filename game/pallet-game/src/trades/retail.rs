use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::{BalanceStatus, ExistenceRequirement},
};
use gafi_support::game::{Amount, Package, Retail, TradeType};
use sp_runtime::Saturating;

impl<T: Config<I>, I: 'static>
	Retail<T::AccountId, T::CollectionId, T::ItemId, T::TradeId, BalanceOf<T, I>, BlockNumber<T>> for Pallet<T, I>
{
	fn do_set_price(
		trade: &T::TradeId,
		who: &T::AccountId,
		package: Package<T::CollectionId, T::ItemId>,
		unit_price: BalanceOf<T, I>,
		start_block: Option<T::BlockNumber>,
		end_block: Option<T::BlockNumber>,
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
		Self::reserved_item(who, &package.collection, &package.item, package.amount)?;

		<BundleOf<T, I>>::try_mutate(trade, |package_vec| -> DispatchResult {
			package_vec
				.try_push(package.clone())
				.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
			Ok(())
		})?;

		TradeConfigOf::<T, I>::insert(
			trade,
			TradeConfig {
				trade: TradeType::SetPrice,
				owner: who.clone(),
				maybe_price: Some(unit_price),
				maybe_required: None,
				start_block,
				end_block,
			},
		);

		Self::deposit_event(Event::<T, I>::PriceSet {
			trade: *trade,
			who: who.clone(),
			collection: package.collection,
			item: package.item,
			amount: package.amount,
			unit_price,
		});

		Ok(())
	}

	// SBP-M2: Try to incorporate safe math operations.
	fn do_buy_item(
		trade: &T::TradeId,
		who: &T::AccountId,
		amount: Amount,
		bid_unit_price: BalanceOf<T, I>,
	) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(trade) {
			ensure!(
				config.trade == TradeType::SetPrice,
				Error::<T, I>::NotSetPrice
			);

			let block_number = <frame_system::Pallet<T>>::block_number();
			if let Some(start_block) = config.start_block {
				ensure!(block_number >= start_block, Error::<T, I>::TradeNotStarted);
			}
			if let Some(end_block) = config.end_block {
				ensure!(block_number <= end_block, Error::<T, I>::TradeEnded);
			}

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
				ensure!(bid_unit_price >= price, Error::<T, I>::BidTooLow);

				// make deposit
				<T as pallet::Config<I>>::Currency::transfer(
					&who,
					&config.owner,
					price.saturating_mul(amount.into()),
					ExistenceRequirement::KeepAlive,
				)?;

				// transfer item
				Self::repatriate_reserved_item(
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
					who: who.clone(),
					amount,
					bid_unit_price,
				});
				return Ok(())
			}
		}

		return Err(Error::<T, I>::UnknownTrade.into())
	}

	fn do_cancel_price(trade: &T::TradeId, who: &T::AccountId) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(trade) {
			ensure!(
				config.trade == TradeType::SetPrice,
				Error::<T, I>::NotSetPrice
			);

			if let Some(package) = BundleOf::<T, I>::get(trade).first() {
				// ensure owner
				ensure!(who.eq(&config.owner), Error::<T, I>::NoPermission);

				// unlock items
				Self::unreserved_item(who, &package.collection, &package.item, package.amount)?;

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

	fn do_add_retail_supply(
		trade: &T::TradeId,
		who: &T::AccountId,
		supply: Package<T::CollectionId, T::ItemId>,
	) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(trade) {
			// check owner
			ensure!(config.owner.eq(who), Error::<T, I>::NoPermission);

			ensure!(
				config.trade == TradeType::SetPrice,
				Error::<T, I>::NotSetPrice
			);

			if let Some(package) = BundleOf::<T, I>::get(trade).first() {
				ensure!(
					package.collection == supply.collection,
					Error::<T, I>::IncorrectCollection
				);
				ensure!(package.item == supply.item, Error::<T, I>::IncorrectItem);

				// ensure transferable
				ensure!(
					T::Nfts::can_transfer(&package.collection, &package.item),
					Error::<T, I>::ItemLocked
				);

				// lock sale items
				Self::reserved_item(who, &package.collection, &package.item, package.amount)?;

				let new_package = Package::new(
					package.collection,
					package.item,
					package.amount + supply.amount,
				);

				<BundleOf<T, I>>::try_mutate(trade, |package_vec| -> DispatchResult {
					*package_vec = BundleFor::<T, I>::try_from([new_package].to_vec())
						.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
					Ok(())
				})?;

				return Ok(())
			}
		}
		return Err(Error::<T, I>::UnknownTrade.into())
	}

	fn do_set_buy(
		trade: &T::TradeId,
		who: &T::AccountId,
		package: Package<T::CollectionId, T::ItemId>,
		unit_price: BalanceOf<T, I>,
		start_block: Option<T::BlockNumber>,
		end_block: Option<T::BlockNumber>,
	) -> DispatchResult {
		// ensure available trade
		ensure!(
			!BundleOf::<T, I>::contains_key(trade),
			Error::<T, I>::TradeIdInUse
		);

		// ensure reserve deposit
		<T as Config<I>>::Currency::reserve(&who, T::BundleDeposit::get())?;

		let deposit = unit_price * package.amount.into();
		<T as Config<I>>::Currency::reserve(&who, deposit)?;

		<BundleOf<T, I>>::try_mutate(trade, |package_vec| -> DispatchResult {
			package_vec
				.try_push(package.clone())
				.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
			Ok(())
		})?;

		TradeConfigOf::<T, I>::insert(
			trade,
			TradeConfig {
				trade: TradeType::SetBuy,
				owner: who.clone(),
				maybe_price: Some(unit_price),
				maybe_required: None,
				start_block,
				end_block,
			},
		);

		Self::deposit_event(Event::<T, I>::BuySet {
			trade: *trade,
			who: who.clone(),
			collection: package.collection,
			item: package.item,
			amount: package.amount,
			unit_price,
		});

		Ok(())
	}

	fn do_claim_set_buy(
		trade: &T::TradeId,
		who: &T::AccountId,
		amount: Amount,
		ask_unit_price: BalanceOf<T, I>,
	) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(trade) {
			ensure!(config.trade == TradeType::SetBuy, Error::<T, I>::NotSetBuy);
			
			let block_number = <frame_system::Pallet<T>>::block_number();
			if let Some(start_block) = config.start_block {
				ensure!(block_number >= start_block, Error::<T, I>::TradeNotStarted);
			}
			if let Some(end_block) = config.end_block {
				ensure!(block_number <= end_block, Error::<T, I>::TradeEnded);
			}

			if let Some(package) = BundleOf::<T, I>::get(trade).first() {
				// ensure item can be transfer
				ensure!(
					T::Nfts::can_transfer(&package.collection, &package.item),
					Error::<T, I>::ItemLocked
				);

				ensure!(package.amount >= amount, Error::<T, I>::SoldOut);

				// check price
				let price = config.maybe_price.unwrap_or_default();
				ensure!(ask_unit_price <= price, Error::<T, I>::AskTooHigh);

				// transfer item
				Self::transfer_item(
					who,
					&package.collection,
					&package.item,
					&config.owner,
					amount,
				)?;

				// make deposit
				<T as pallet::Config<I>>::Currency::repatriate_reserved(
					&config.owner,
					&who,
					price * amount.into(),
					BalanceStatus::Free,
				)?;

				let new_package =
					Package::new(package.collection, package.item, package.amount - amount);

				<BundleOf<T, I>>::try_mutate(trade, |package_vec| -> DispatchResult {
					*package_vec = BundleFor::<T, I>::try_from([new_package].to_vec())
						.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
					Ok(())
				})?;

				Self::deposit_event(Event::<T, I>::SetBuyClaimed {
					trade: *trade,
					who: who.clone(),
					amount: package.amount,
					ask_unit_price,
				});
				return Ok(())
			}
		}
		return Err(Error::<T, I>::UnknownTrade.into())
	}

	fn do_cancel_set_buy(trade: &T::TradeId, who: &T::AccountId) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(trade) {
			ensure!(config.trade == TradeType::SetBuy, Error::<T, I>::NotSetBuy);

			if let Some(package) = BundleOf::<T, I>::get(trade).first() {
				// ensure owner
				ensure!(who.eq(&config.owner), Error::<T, I>::NoPermission);

				// unreserve deposit
				let price = config.maybe_price.unwrap_or_default();

				<T as pallet::Config<I>>::Currency::unreserve(
					&config.owner,
					price.saturating_mul(package.amount.into()),
				);

				// end trade
				<T as pallet::Config<I>>::Currency::unreserve(
					&config.owner,
					T::BundleDeposit::get(),
				);
				BundleOf::<T, I>::remove(trade);
				TradeConfigOf::<T, I>::remove(trade);

				return Ok(())
			}
		}
		Err(Error::<T, I>::UnknownTrade.into())
	}
}
