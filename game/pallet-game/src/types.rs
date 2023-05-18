use codec::{Decode, Encode, MaxEncodedLen};
use core::primitive::u32;
use frame_support::{RuntimeDebug};
use pallet_nfts::{CollectionSettings, MintSettings, CollectionConfig};
use scale_info::TypeInfo;

/// Information about a game.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(NameLimit))]
pub struct GameDetails<AccountId, DepositBalance> {
	/// game's owner.
	pub(super) owner: AccountId,
	/// The total balance deposited by the owner for all the storage data associated with this
	/// game. Used by `destroy`.
	pub(super) owner_deposit: DepositBalance,
	/// The total number of outstanding collections of this game.
	pub(super) collections: u32,
	/// Can thaw tokens, force transfers and burn tokens from any account.
	pub(super) admin: AccountId,
}

// /// Holds the information about minting.
// #[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
// pub struct GameMintSettings<Price, BlockNumber, CollectionId>{
// 	/// Default settings each item will get during the mint.
// 	pub mint_settings: MintSettings<Price, BlockNumber, CollectionId>,
// }

// impl<Price, BlockNumber, CollectionId> Default
// 	for GameMintSettings<Price, BlockNumber, CollectionId>
// {
// 	fn default() -> Self {
// 		Self {
// 			mint_settings: MintSettings::default(),
// 		}
// 	}
// }

// /// Game Collection's configuration.
// #[derive(
// 	Clone, Copy, Decode, Default, Encode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo,
// )]
// pub struct GameCollectionConfig<Price, BlockNumber, CollectionId> {
// 	/// Collection's settings.
// 	pub settings: CollectionSettings,
// 	/// Collection's max supply.
// 	pub max_supply: Option<u32>,
// 	/// Default settings each item will get during the mint.
// 	pub mint_settings: GameMintSettings<Price, BlockNumber, CollectionId>,
// }

// impl<Price, BlockNumber, CollectionId> GameCollectionConfig<Price, BlockNumber, CollectionId> {
// 	pub fn to_collection_config(self) -> CollectionConfig<Price, BlockNumber, CollectionId> {
// 		CollectionConfig {
// 			mint_settings: self.mint_settings.mint_settings,
// 			max_supply: self.max_supply,
// 			settings: self.settings,
// 		}
// 	}
// }


#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(NameLimit))]
pub struct Item<ItemId> {
	pub item: ItemId,
	pub amount: u32,
}

impl<ItemId> Item<ItemId> {
	pub fn new(item: ItemId, amount: u32) -> Self {
		Item { item, amount }
	}
}

/// Item upgrade configuration.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ItemUpgradeConfig<ItemId, Price> {

	pub item: ItemId,

	pub fee: Price,
}
