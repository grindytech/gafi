use crate::*;
use frame_support::{pallet_prelude::*, traits::BalanceStatus};
use frame_system::pallet_prelude::BlockNumberFor;
use gafi_support::game::{Auction, Bundle};
use sp_runtime::Saturating;

impl<T: Config<I>, I: 'static>
	Auction<
		T::AccountId,
		T::CollectionId,
		T::ItemId,
		T::TradeId,
		BalanceOf<T, I>,
		BlockNumberFor<T>,
	> for Pallet<T, I>
{
	fn do_set_auction(
		trade: &T::TradeId,
		who: &T::AccountId,
		source: Bundle<T::CollectionId, T::ItemId>,
		maybe_price: Option<BalanceOf<T, I>>,
		start_block: Option<BlockNumberFor<T>>,
		duration: BlockNumberFor<T>,
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

		let start = match start_block {
			Some(block) => block,
			None => <frame_system::Pallet<T>>::block_number(),
		};

		AuctionConfigOf::<T, I>::insert(
			trade,
			AuctionConfig {
				owner: who.clone(),
				maybe_price,
				start_block: start,
				duration,
			},
		);

		Self::deposit_event(Event::<T, I>::AuctionSet {
			trade: *trade,
			who: who.clone(),
			source,
			maybe_price,
			start_block,
			duration,
		});

		Ok(())
	}

	fn do_bid_auction(
		trade: &T::TradeId,
		who: &T::AccountId,
		bid: BalanceOf<T, I>,
	) -> DispatchResult {
		if let Some(config) = AuctionConfigOf::<T, I>::get(trade) {
			// make sure the auction is not over
			let block_number = <frame_system::Pallet<T>>::block_number();
			ensure!(
				block_number >= config.start_block,
				Error::<T, I>::AuctionNotStarted
			);
			ensure!(
				block_number < config.start_block.saturating_add(config.duration),
				Error::<T, I>::AuctionEnded
			);

			if let Some(price) = config.maybe_price {
				ensure!(bid >= price, Error::<T, I>::BidTooLow);
			}
			// update winner
			if let Some(highest_bid) = HighestBidOf::<T, I>::get(trade) {
				ensure!(bid > highest_bid.1, Error::<T, I>::BidTooLow);
				<T as Config<I>>::Currency::unreserve(&highest_bid.0, highest_bid.1);
			}

			HighestBidOf::<T, I>::insert(trade, (who, bid));
			<T as Config<I>>::Currency::reserve(&who, bid)?;

			Self::deposit_event(Event::<T, I>::Bid {
				trade: *trade,
				who: who.clone(),
				bid,
			});
			return Ok(())
		}
		Err(Error::<T, I>::UnknownAuction.into())
	}

	fn do_claim_auction(trade: &T::TradeId) -> DispatchResult {
		if let Some(config) = AuctionConfigOf::<T, I>::get(trade) {
			let block_number = <frame_system::Pallet<T>>::block_number();

			ensure!(
				block_number >= (config.start_block.saturating_add(config.duration)),
				Error::<T, I>::AuctionInProgress
			);
			let maybe_bid = HighestBidOf::<T, I>::get(trade);
			if let Some(highest_bid) = maybe_bid.clone() {
				if let Some(auction) = AuctionConfigOf::<T, I>::get(trade) {
					<T as pallet::Config<I>>::Currency::repatriate_reserved(
						&highest_bid.0,
						&auction.owner,
						highest_bid.1,
						BalanceStatus::Free,
					)?;

					for package in BundleOf::<T, I>::get(trade) {
						Self::repatriate_reserved_item(
							&auction.owner,
							&package.collection,
							&package.item,
							&highest_bid.0,
							package.amount,
							ItemBalanceStatus::Free,
						)?;
					}

					<T as Config<I>>::Currency::unreserve(&auction.owner, T::BundleDeposit::get());
				}
			}
			AuctionConfigOf::<T, I>::remove(trade);
			BundleOf::<T, I>::remove(trade);
			HighestBidOf::<T, I>::remove(trade);

			Self::deposit_event(Event::<T, I>::AuctionClaimed {
				trade: *trade,
				maybe_bid,
			});
			return Ok(())
		}
		Err(Error::<T, I>::UnknownAuction.into())
	}
}
