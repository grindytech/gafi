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
		todo!()
	}

	fn do_fill_wishlist(
		id: &T::TradeId,
		who: &T::AccountId,
		ask_price: BalanceOf<T, I>,
	) -> DispatchResult {
		todo!()
	}
}
