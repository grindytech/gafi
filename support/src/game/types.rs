use codec::{Decode, Encode};
use core::primitive::u32;
use frame_support::{pallet_prelude::MaxEncodedLen, RuntimeDebug};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use super::Amount;

pub type Bundle<CollectionId, ItemId> = Vec<Package<CollectionId, ItemId>>;
pub type Distribution<CollectionId, ItemId> = Vec<Fraction<CollectionId, ItemId>>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct Package<CollectionId, ItemId> {
	pub collection: CollectionId,
	pub item: ItemId,
	pub amount: u32,
}


impl<CollectionId, ItemId> Package<CollectionId, ItemId> {
	pub fn new(collection: CollectionId, item: ItemId, amount: Amount) -> Self {
		Package {
			collection,
			item,
			amount,
		}
	}
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct Fraction<CollectionId, ItemId> {
	pub collection: CollectionId,
	pub item: ItemId,
	pub permil: u32,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TradeType {
	SetPrice,
	SetBuy,
	Bundle,
	Wishlist,
	Auction,
	Swap,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum PoolType {
	Dynamic,
 	Fixed,
}
