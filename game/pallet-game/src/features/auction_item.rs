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
		if let Some(config) = AuctionConfigOf::<T, I>::get(id) {
			// make sure the auction is not over
			let block_number = <frame_system::Pallet<T>>::block_number();
			ensure!(
				block_number >= config.start_block,
				Error::<T, I>::AuctionNotStarted
			);
			ensure!(
				block_number < config.start_block + config.duration,
				Error::<T, I>::AuctionEnded
			);

			if let Some(price) = config.maybe_price {
				ensure!(bid >= price, Error::<T, I>::BidTooLow);
			}
			// update winner
			if let Some(winner_bid) = BidWinnerOf::<T, I>::get(id) {
				ensure!(bid > winner_bid.1, Error::<T, I>::BidTooLow);
				<T as Config<I>>::Currency::unreserve(&winner_bid.0, winner_bid.1);
			}

			BidWinnerOf::<T, I>::insert(id, (who, bid));
			<T as Config<I>>::Currency::reserve(&who, bid)?;
			return Ok(())
		}
		Err(Error::<T, I>::UnknownAuction.into())
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

					<T as Config<I>>::Currency::unreserve(&auction.owner, T::BundleDeposit::get());
				}
			}
			AuctionConfigOf::<T, I>::remove(id);
			BundleOf::<T, I>::remove(id);
			return Ok(())
		}
		Err(Error::<T, I>::UnknownAuction.into())
	}
}
