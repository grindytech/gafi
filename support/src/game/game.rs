use sp_std::vec::Vec;
use crate::common::types::{BlockNumber};

pub trait GameSetting< E, AccountId, GameId> {
	fn create_game(id: GameId, owner: AccountId, admin: Option<AccountId>, name: Vec<u8>) -> Result<GameId, E>;
	fn set_swapping_fee(id: GameId, fee: u8, start_block: BlockNumber) -> Result<(), E>;
}

pub trait GameNfts<E, AccountId, GameId, CollectionId, ItemId, Attribute, Balance> {
	fn create_game_collection(game_id: GameId, collection_id: CollectionId) -> Result<CollectionId, E>;
	fn create_collection(collection_id: CollectionId, admin: AccountId) -> Result<CollectionId, E>;
	fn create_item(collection_id: CollectionId, item_id: ItemId)-> Result<ItemId, E>;
	fn add_item(collection_id: CollectionId, item_id: ItemId, amount: u32) -> Result<ItemId, E>;
	fn mint(collection_id: CollectionId) -> Result<ItemId, E>;
	fn set_upgrade(item_id: ItemId,  attribute: Attribute, level: u8, fee: Balance) -> Result<(), E>;
	fn upgrade(item_id: ItemId)-> Result<(), E>;
	
	fn transfer(target: AccountId, item_id: ItemId, amount: u32) -> Result<(), E>;
	fn burn();
	fn swap();
}
