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
		config: &TradeConfig<BalanceOf<T, I>>,
	) -> DispatchResult {
		// ensure balance
		ensure!(
			ItemBalances::<T, I>::get((who, collection, item)) >= config.amount,
			Error::<T, I>::InsufficientItemBalance
		);

		// ensure transferable
		ensure!(
			T::Nfts::can_transfer(collection, item),
			Error::<T, I>::ItemLocked
		);

		TradeConfigOf::<T, I>::insert((who, collection, item), config);

		Ok(())
	}
}
