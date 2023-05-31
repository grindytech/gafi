use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::game::{Amount, TransferItem, Swap};

impl<T: Config<I>, I: 'static> Swap<T::AccountId, T::CollectionId, T::ItemId, T::TradeId, BalanceOf<T, I>>
	for Pallet<T, I>
{
    fn do_set_swap(
		    id: &T::TradeId,
		    who: &T::AccountId,
		    source: gafi_support::game::Bundle<T::CollectionId, T::ItemId>,
		    required: gafi_support::game::Bundle<T::CollectionId, T::ItemId>,
		    maybe_price: Option<BalanceOf<T, I>>,
	    ) -> DispatchResult {
        todo!()
    }

    fn do_claim_swap(
		    id: &T::TradeId,
		    who: &T::AccountId,
		    maybe_bid_price: Option<BalanceOf<T, I>>,
	    ) -> DispatchResult {
        todo!()
    }
}
