use codec::{Decode, Encode};
use core::primitive::u32;
use frame_support::{
	pallet_prelude::{BoundedVec, MaxEncodedLen},
	traits::Get,
	RuntimeDebug,
};
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

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct Package<CollectionId, ItemId> {
	pub(super) collection: CollectionId,
	pub(super) item: ItemId,
	pub(super) amount: u32,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(PackageLimit))]
pub struct Bundle<CollectionId, ItemId, Price, PackageLimit: Get<u32>> {
	pub(super) packages: BoundedVec<Package<CollectionId, ItemId>, PackageLimit>,
	pub(super) price: Price,
}
