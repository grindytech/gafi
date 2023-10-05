use codec::{Decode, Encode};
use core::primitive::u32;
use frame_support::{pallet_prelude::MaxEncodedLen, RuntimeDebug};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use sp_runtime::traits::Printable;
use sp_std::fmt::{Debug, Formatter};

use super::Amount;

pub type Bundle<CollectionId, ItemId> = Vec<Package<CollectionId, ItemId>>;
pub type LootTable<CollectionId, ItemId> = Vec<Loot<CollectionId, ItemId>>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct Package<CollectionId, ItemId> {
	pub collection: CollectionId,
	pub item: ItemId,
	pub amount: Amount,
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

/// Payload used to hold seed data required to submit a transaction.
#[derive(Encode, Decode, Clone, PartialEq, Eq, scale_info::TypeInfo, MaxEncodedLen)]
pub struct SeedPayload<BlockNumber, Seed> {
	pub block_number: BlockNumber,
	pub seed: Seed,
}

impl<BlockNumber, Seed> Printable for SeedPayload<BlockNumber, Seed>
where
	BlockNumber: Debug,
	Seed: Debug,
{
	fn print(&self) {
		let block_number = &self.block_number;
		let seed = &self.seed;
		log::info!(
			"Current random seed: block_number = {:?}, seed = {:?}",
			block_number,
			seed
		);
	}
}

impl<BlockNumber, Seed> Debug for SeedPayload<BlockNumber, Seed>
where
	BlockNumber: Debug,
	Seed: Debug + AsRef<[u8]>,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> sp_std::fmt::Result {
		write!(f, "    block_number: {:?},\n", self.block_number)?;
		if let Ok(seed) = sp_std::str::from_utf8(self.seed.as_ref()) {
			write!(f, "    seed: {:?}\n", seed)?;
		} else {
			write!(f, "    seed: {:?}\n", self.seed)?;
		}
		Ok(())
	}
}
