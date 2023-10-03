use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use frame_system::pallet_prelude::BlockNumberFor;
use gafi_support::game::{Bundle, Swap, TradeType};

impl<T: Config<I>, I: 'static>
	Swap<T::AccountId, T::CollectionId, T::ItemId, T::TradeId, BalanceOf<T, I>, BlockNumberFor<T>>
	for Pallet<T, I>
{
	fn do_set_swap(
		trade: &T::TradeId,
		who: &T::AccountId,
		source: Bundle<T::CollectionId, T::ItemId>,
		required: Bundle<T::CollectionId, T::ItemId>,
		maybe_price: Option<BalanceOf<T, I>>,
		start_block: Option<BlockNumberFor<T>>,
		end_block: Option<BlockNumberFor<T>>,
	) -> DispatchResult {
		// ensure available trade
		ensure!(
			!BundleOf::<T, I>::contains_key(trade),
			Error::<T, I>::TradeIdInUse,
		);

		<T as Config<I>>::Currency::reserve(&who, T::BundleDeposit::get())?;

		// lock bundle
		for package in source.clone() {
			Self::reserved_item(who, &package.collection, &package.item, package.amount)?;
		}

		<BundleOf<T, I>>::try_mutate(trade, |package_vec| -> DispatchResult {
			package_vec
				.try_append(source.clone().into_mut())
				.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
			Ok(())
		})?;

		let bundle_out: BundleFor<T, I> =
			BoundedVec::try_from(required.clone()).map_err(|_| Error::<T, I>::ExceedMaxBundle)?;

		TradeConfigOf::<T, I>::insert(
			trade,
			TradeConfig {
				trade: TradeType::Swap,
				owner: who.clone(),
				maybe_price,
				maybe_required: Some(bundle_out),
				start_block,
				end_block,
			},
		);

		Self::deposit_event(Event::<T, I>::SwapSet {
			trade: *trade,
			who: who.clone(),
			source,
			required,
			maybe_price,
		});

		Ok(())
	}

	fn do_claim_swap(
		trade: &T::TradeId,
		who: &T::AccountId,
		maybe_bid_price: Option<BalanceOf<T, I>>,
	) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(trade) {
			ensure!(config.trade == TradeType::Swap, Error::<T, I>::NotSwap);

			let block_number = <frame_system::Pallet<T>>::block_number();
			if let Some(start_block) = config.start_block {
				ensure!(block_number >= start_block, Error::<T, I>::TradeNotStarted);
			}
			if let Some(end_block) = config.end_block {
				ensure!(block_number <= end_block, Error::<T, I>::TradeEnded);
			}

			if let Some(price) = config.maybe_price {
				// check price
				ensure!(
					maybe_bid_price.unwrap_or_default() >= price,
					Error::<T, I>::BidTooLow
				);

				// make deposit
				<T as pallet::Config<I>>::Currency::transfer(
					&who,
					&config.owner,
					price,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			// transfer items
			if let Some(required) = config.maybe_required {
				for package in required.clone() {
					Self::transfer_item(
						&who,
						&package.collection,
						&package.item,
						&config.owner,
						package.amount,
					)?;
				}
			}

			for package in BundleOf::<T, I>::get(trade).clone() {
				Self::repatriate_reserved_item(
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

			Self::deposit_event(Event::<T, I>::SwapClaimed {
				trade: *trade,
				who: who.clone(),
				maybe_bid_price,
			});
			return Ok(())
		}
		Err(Error::<T, I>::UnknownTrade.into())
	}

	fn do_cancel_swap(trade: &T::TradeId, who: &T::AccountId) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(trade) {
			ensure!(config.trade == TradeType::Swap, Error::<T, I>::NotSwap);

			// ensure owner
			ensure!(who.eq(&config.owner), Error::<T, I>::NoPermission);

			let bundle = BundleOf::<T, I>::get(trade);
			// unlock items
			for package in bundle.clone() {
				Self::unreserved_item(who, &package.collection, &package.item, package.amount)?;
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
}
