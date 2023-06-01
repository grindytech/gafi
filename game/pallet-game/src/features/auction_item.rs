use crate::*;
use frame_support::{pallet_prelude::*, traits::BalanceStatus};
use gafi_support::game::{Auction, Bundle};

impl<T: Config<I>, I: 'static>
	Auction<T::AccountId, T::CollectionId, T::ItemId, T::TradeId, BalanceOf<T, I>, T::BlockNumber>
	for Pallet<T, I>
{
	fn do_set_auction(
		id: &T::TradeId,
		who: &T::AccountId,
		source: Bundle<T::CollectionId, T::ItemId>,
		maybe_price: Option<BalanceOf<T, I>>,
		start_block: T::BlockNumber,
		duration: T::BlockNumber,
	) -> DispatchResult {
		// ensure available id
		ensure!(
			!BundleOf::<T, I>::contains_key(id),
			Error::<T, I>::TradeIdInUse,
		);

		<T as Config<I>>::Currency::reserve(&who, T::BundleDeposit::get())?;

		// lock bundle
		for package in source.clone() {
			Self::lock_item(who, &package.collection, &package.item, package.amount)?;
		}

		<BundleOf<T, I>>::try_mutate(id, |package_vec| -> DispatchResult {
			package_vec
				.try_append(source.clone().into_mut())
				.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
			Ok(())
		})?;

		NextTradeId::<T, I>::set(Some(id.increment()));

		AuctionConfigOf::<T, I>::insert(
			id,
			AuctionConfig {
				owner: who.clone(),
				maybe_price,
				start_block,
				duration,
			},
		);

		Ok(())
	}

	fn do_bid_auction(id: &T::TradeId, who: &T::AccountId, bid: BalanceOf<T, I>) -> DispatchResult {
		if let Some(cofig) = AuctionConfigOf::<T, I>::get(id) {
			let total_bid = bid + BidPriceOf::<T, I>::get(id, who).unwrap_or_default();

			if let Some(price) = cofig.maybe_price {
				ensure!(total_bid >= price, Error::<T, I>::BidTooLow);
			}

			// check bid amount is exists
			ensure!(
				!BidderOf::<T, I>::contains_key(id, total_bid),
				Error::<T, I>::BidExists
			);

			<T as Config<I>>::Currency::reserve(&who, bid)?;

			// update winner
			if let Some(winner_bid) = BidWinnerOf::<T, I>::get(id) {
				if total_bid > winner_bid.1 {
					BidWinnerOf::<T, I>::insert(id, (who, total_bid));
				}
			} else {
				BidWinnerOf::<T, I>::insert(id, (who, total_bid));
			}

			BidderOf::<T, I>::insert(id, total_bid, who.clone());
			BidPriceOf::<T, I>::insert(id, who, total_bid);
		}

		Ok(())
	}

	fn fn_cancel_bid(id: &T::TradeId, who: &T::AccountId) -> DispatchResult {
		todo!()
	}

	fn do_set_candle_auction(
		id: &T::TradeId,
		who: &T::AccountId,
		bundle: Bundle<T::CollectionId, T::ItemId>,
		maybe_price: Option<BalanceOf<T, I>>,
		start_block: T::BlockNumber,
		early_end: T::BlockNumber,
		end_block: T::BlockNumber,
	) -> DispatchResult {
		todo!()
	}

	fn do_claim_auction(id: &T::TradeId) -> DispatchResult {
		if let Some(config) = AuctionConfigOf::<T, I>::get(id) {
			let block_number = <frame_system::Pallet<T>>::block_number();

			ensure!(
				block_number >= (config.start_block + config.duration),
				Error::<T, I>::AuctionInProgress
			);

			if let Some(winner_bid) = BidWinnerOf::<T, I>::get(id) {
				if let Some(auction) = AuctionConfigOf::<T, I>::get(id) {
					<T as pallet::Config<I>>::Currency::repatriate_reserved(
						&winner_bid.0,
						&auction.owner,
						winner_bid.1,
						BalanceStatus::Free,
					)?;

					for package in BundleOf::<T, I>::get(id) {
						Self::repatriate_lock_item(
							&auction.owner,
							&package.collection,
							&package.item,
							&winner_bid.0,
							package.amount,
							ItemBalanceStatus::Free,
						)?;
					}
				}
			}
			// make the trade
			return Ok(())
		}

		Err(Error::<T, I>::UnknownAuction.into())
	}
}
