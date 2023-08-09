use codec::{Decode, Encode};
use core::primitive::u32;
use frame_support::{pallet_prelude::MaxEncodedLen, RuntimeDebug};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use super::Amount;

pub type Bundle<CollectionId, ItemId> = Vec<Package<CollectionId, ItemId>>;
pub type LootTable<CollectionId, ItemId> = Vec<Loot<CollectionId, ItemId>>;

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

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TradeType {
	SetPrice,
	SetBuy,
	Bundle,
	Wishlist,
	Auction,
	Swap,
}

/// Types of the minting pool
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum PoolType {
	/// Item minting chance will change depending on item supply.
	Dynamic,
	/// Item minting chance is fixed with an infinite supply.
	Stable,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct NFT<CollectionId, ItemId> {
	pub collection: CollectionId,
	pub item: ItemId,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct Loot<CollectionId, ItemId> {
	/// Each loot can be an nft or nothing
	pub maybe_nft: Option<NFT<CollectionId, ItemId>>,
	pub weight: u32,
}

#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MintType<CollectionId> {
	/// Anyone could mint items.
	Public,
	/// Only holders of items in specified collection could mint new items.
	HolderOf(CollectionId),
}

/// Holds the information about minting.
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MintSettings<Price, BlockNumber, CollectionId> {
	/// Whether anyone can mint or if minters are restricted to some subset.
	pub mint_type: MintType<CollectionId>,
	/// An price per mint.
	pub price: Price,
	/// When the mint starts.
	pub start_block: Option<BlockNumber>,
	/// When the mint ends.
	pub end_block: Option<BlockNumber>,
}
