use sp_std::vec::Vec;
use crate::common::types::BlockNumber;

pub trait GameSetting< E, AccountId, GameId> {
	fn create_game(id: GameId, owner: AccountId, name: Vec<u8>) -> Result<GameId, E>;
	fn set_swapping_fee(id: GameId, fee: u8, start_block: BlockNumber) -> Result<(), E>;
	fn freeze_collection_transfer();
	fn freeze_collection_swap();
	fn freeze_item_transfer();
	fn freeze_item_swap();
}

pub trait GameProvider<E, AccountId, GameId> {
	fn get_swap_fee() -> Result<u8, E>;
	fn is_game_owner(id: GameId, owner: AccountId) -> Result<(), E>;
}


pub trait GameNfts {
	fn create_collection();
	fn create_item();
	fn mint();
	fn set_upgrade();
	fn upgrade();
	fn swap();
	fn transfer();
	fn burn();
}

pub trait GameNftsProvider {
	fn is_collection_owner();
	fn is_collection_admin();
}