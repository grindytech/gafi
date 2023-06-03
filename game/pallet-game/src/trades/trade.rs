use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::game::{Retail, Trade, TradeType};

impl<T: Config<I>, I: 'static> Trade<T::AccountId, T::TradeId> for Pallet<T, I> {
	fn do_cancel_trade(
		trade: &T::TradeId,
		who: &T::AccountId,
		trade_type: TradeType,
	) -> DispatchResult {
		match trade_type {
			TradeType::SetPrice => {
				Self::do_cancel_price(trade, who)?;
			},
			TradeType::Bundle => {
				Self::do_cancel_bundle(trade, who)?;
			},
			TradeType::Wishlist => {
				Self::do_cancel_wishlist(trade, who)?;
			},
			TradeType::Auction => {
				Self::do_claim_auction(trade)?;
			},
			TradeType::Swap => {
				Self::do_cancel_swap(trade, who)?;
			},
			_ => return Err(Error::<T, I>::UnknownTrade.into()),
		};
		Err(Error::<T, I>::UnknownTrade.into())
	}
}
