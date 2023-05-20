use codec::{Decode, Encode, MaxEncodedLen};
use core::primitive::u32;
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

use super::Amount;

/// Trade Item configuration.
/// - `price`: price of each item, `None` for canceled sell
/// - `amount`: amount of items
/// - `min_order_quantity`: Minimum Order Quantity, None is selling all or nothing.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct TradeConfig<Price> {
	pub price: Price,
	pub amount: Amount,
	pub min_order_quantity: Option<u32>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Package<CollectionId, ItemId> {
	pub collection: CollectionId,
	pub item: ItemId,
	pub amount: Amount,
}
