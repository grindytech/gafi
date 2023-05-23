use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, Bundle, Package, Trade, Wishlist};

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
		todo!()
	}
}
