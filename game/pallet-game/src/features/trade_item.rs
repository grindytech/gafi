use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::game::Trade;

impl<T: Config<I>, I: 'static> Trade<T::AccountId, T::CollectionId, T::ItemId, BalanceOf<T, I>>
	for Pallet<T, I>
{
	fn do_set_price(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		price: Option<BalanceOf<T, I>>,
		amount: gafi_support::game::Amount,
		moq: Option<u32>,
	) {
		todo!()
	}
}
