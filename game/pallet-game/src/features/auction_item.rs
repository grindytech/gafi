use crate::*;
use frame_support::pallet_prelude::*;
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
			let total_bid = TotalBidOf::<T, I>::get(who, id).unwrap_or_default();
			
			if let Some(price) = cofig.maybe_price {
				ensure!(total_bid + bid >= price, Error::<T, I>::BidTooLow);
			}

			<T as Config<I>>::Currency::reserve(&who, bid)?;
			let block_number = <frame_system::Pallet<T>>::block_number();

			BidOf::<T, I>::try_mutate(who, id, |bid_vec| -> DispatchResult {
				bid_vec
					.try_push((block_number, bid))
					.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
				Ok(())
			})?;

			TotalBidOf::<T, I>::insert(who, id, total_bid + bid);
		}

		Ok(())
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

	fn fn_cancel_bid(id: &T::TradeId, who: &T::AccountId) -> DispatchResult {
		todo!()
	}
}
