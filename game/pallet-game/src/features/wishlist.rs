use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::{BalanceStatus},
};
use gafi_support::game::{Bundle, Wishlist};

impl<T: Config<I>, I: 'static>
	Wishlist<T::AccountId, T::CollectionId, T::ItemId, T::TradeId, BalanceOf<T, I>> for Pallet<T, I>
{
	fn do_set_wishlist(
		id: &T::TradeId,
		who: &T::AccountId,
		bundle: Bundle<T::CollectionId, T::ItemId>,
		price: BalanceOf<T, I>,
	) -> DispatchResult {
		// ensure available id
		ensure!(!BundleOf::<T, I>::contains_key(id), Error::<T, I>::IdExists);

		<T as Config<I>>::Currency::reserve(&who, T::BundleDeposit::get())?;
		<T as Config<I>>::Currency::reserve(&who, price)?;

		<BundleOf<T, I>>::try_mutate(id, |package_vec| -> DispatchResult {
			package_vec
				.try_append(bundle.clone().into_mut())
				.map_err(|_| Error::<T, I>::ExceedMaxBundle)?;
			Ok(())
		})?;

		NextTradeId::<T, I>::set(Some(id.increment()));

		TradeConfigOf::<T, I>::insert(
			id,
			TradeConfig {
				trade: TradeType::Wishlist,
				owner: who.clone(),
				price,
			},
		);

		Ok(())
	}

	fn do_fill_wishlist(
		id: &T::TradeId,
		who: &T::AccountId,
		ask_price: BalanceOf<T, I>,
	) -> DispatchResult {
		let bundle = BundleOf::<T, I>::get(id);

		// ensure item can be transfer
		for pack in bundle.clone() {
			ensure!(
				T::Nfts::can_transfer(&pack.collection, &pack.item),
				Error::<T, I>::ItemLocked
			);
		}

		if let Some(config) = TradeConfigOf::<T, I>::get(id) {
			ensure!(config.trade == TradeType::Wishlist, Error::<T, I>::UnknownTrade);

			// check price
			ensure!(ask_price <= config.price, Error::<T, I>::AskTooHigh);

			// make deposit
			<T as pallet::Config<I>>::Currency::repatriate_reserved(
				&config.owner,
				&who,
				config.price,
				BalanceStatus::Free,
			)?;

			// transfer items
			for package in bundle.clone() {
				Self::transfer_item(
					&who,
					&package.collection,
					&package.item,
					&config.owner,
					package.amount,
				)?;
			}
			<T as pallet::Config<I>>::Currency::unreserve(&config.owner, T::BundleDeposit::get());
		} else {
			return Err(Error::<T, I>::UnknownTrade.into())
		}

		Ok(())
	}
}
