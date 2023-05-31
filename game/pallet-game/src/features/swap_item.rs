use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Bundle, Swap};

impl<T: Config<I>, I: 'static>
	Swap<T::AccountId, T::CollectionId, T::ItemId, T::TradeId, BalanceOf<T, I>> for Pallet<T, I>
{
	fn do_set_swap(
		id: &T::TradeId,
		who: &T::AccountId,
		source: Bundle<T::CollectionId, T::ItemId>,
		required: Bundle<T::CollectionId, T::ItemId>,
		maybe_price: Option<BalanceOf<T, I>>,
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

		let bundle_out: BundleFor<T, I> =
			BoundedVec::try_from(required).map_err(|_| Error::<T, I>::ExceedMaxBundle)?;

		TradeConfigOf::<T, I>::insert(
			id,
			TradeConfig {
				trade: TradeType::Swap,
				owner: who.clone(),
				maybe_price,
				maybe_required: Some(bundle_out),
			},
		);

		Ok(())
	}

	fn do_claim_swap(
		id: &T::TradeId,
		who: &T::AccountId,
		maybe_bid_price: Option<BalanceOf<T, I>>,
	) -> DispatchResult {
		if let Some(config) = TradeConfigOf::<T, I>::get(id) {
			ensure!(config.trade == TradeType::Swap, Error::<T, I>::UnknownTrade);

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

			for package in BundleOf::<T, I>::get(id).clone() {
				Self::transfer_lock_item(
					&config.owner,
					&package.collection,
					&package.item,
					who,
					package.amount,
				)?;

				Self::unlock_item(who, &package.collection, &package.item, package.amount)?;
			}

			<T as pallet::Config<I>>::Currency::unreserve(&config.owner, T::BundleDeposit::get());

			return Ok(())
		}
		Ok(())
	}
}
