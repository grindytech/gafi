use super::{Amount, Level, Metadata};
use frame_support::pallet_prelude::DispatchResult;
use sp_runtime::{Percent, TokenError};
use sp_std::vec::Vec;

pub trait GameSetting<AccountId, GameId, BlockNumber> {
	/// Do create a new game
	///
	/// Implementing the function create game
	///
	/// Parameters:
	/// - `id`: new game id
	/// - `owner`: owner
	/// - `maybe_admin`: admin
	/// - `maybe_name`: name
	fn do_create_game(
		game_id: GameId,
		who: AccountId,
		maybe_admin: Option<AccountId>,
	) -> DispatchResult;

	/// Do set swap fee
	///
	///  Implementing the function set swap fee
	///
	/// Parameters:
	/// - `id`: game id
	/// - `owner`: owner
	/// - `fee`: percent of swapping volume
	/// - `start_block`: block apply swap fee
	fn do_set_swap_fee(
		game_id: GameId,
		who: AccountId,
		fee: Percent,
		start_block: BlockNumber,
	) -> DispatchResult;
}

pub trait CreateCollection<AccountId, GameId, CollectionId, CollectionConfig> {
	/// Do create game collection
	///
	/// Create collection for specific game
	///
	/// Parameters:
	/// - `who`: signer and collection owner
	/// - `game_id`: game id
	/// - `maybe_admin`: if admin not provided, owner also an admin
	/// - `config`: collection configuration
	fn do_create_game_collection(
		who: AccountId,
		game_id: GameId,
		maybe_admin: Option<AccountId>,
		config: CollectionConfig,
	) -> DispatchResult;

	/// Do create collection
	///
	/// Create a pure collection
	///
	/// Parameters:
	/// - `who`: signer and collection owner
	/// - `maybe_admin`: if admin not provided, owner also an admin
	/// - `config`: collection configuration
	fn do_create_collection(
		who: AccountId,
		maybe_admin: Option<AccountId>,
		config: CollectionConfig,
	) -> DispatchResult;

	fn do_add_collection(
		who: AccountId,
		game_id: GameId,
		collection_ids: Vec<CollectionId>,
	) -> DispatchResult;
}

pub trait CreateItem<AccountId, CollectionId, ItemId, ItemConfig> {
	/// Create item
	///
	/// Create items for collection
	///
	/// Parameters:
	/// - `collection_id`: collection id
	/// - `item_id`: item id
	/// - `amount`: amount
	fn do_create_item(
		who: AccountId,
		collection_id: CollectionId,
		item_id: ItemId,
		config: ItemConfig,
		amount: Amount,
	) -> DispatchResult;

	/// Do add item
	///
	/// Add number amount of item in collection
	///
	/// Parameters:
	/// - `collection_id`: collection id
	/// - `item_id`: item id
	/// - `amount`: amount
	fn do_add_item(
		who: AccountId,
		collection_id: CollectionId,
		item_id: ItemId,
		amount: Amount,
	) -> DispatchResult;
}

pub trait Mutable<AccountId, GameId, CollectionId, ItemId> {
	/// Mint
	///
	/// Random mint item in the collection
	///
	/// Parameters:
	/// - `_who`: sender
	/// - `_collection_id`: collection id
	/// - `_target`: recipient account, default `minter`
	///
	/// By default, this is not a supported operation.
	fn do_mint(
		_who: AccountId,
		_collection_id: CollectionId,
		_target: AccountId,
	) -> DispatchResult {
		Err(TokenError::Unsupported.into())
	}

	/// Burn
	///
	/// Burn item
	///
	/// Parameters:
	/// - `who`: item owner
	/// - `collection_id`: collection id
	/// - `item_id`: item id
	/// - `maybe_amount`: amount of items to burn, default `1`
	fn do_burn(
		who: AccountId,
		collection_id: CollectionId,
		item_id: ItemId,
		maybe_amount: Amount,
	) -> DispatchResult;
}

pub trait Upgrade<AccountId, Balance, CollectionId, ItemId, StringLimit> {
	/// Set Upgrade
	///
	/// Set upgrade item                          
	///
	/// Parameters:
	/// - `who`: item owner
	/// - `collection_id`: collection id
	/// - `item_id`: item id
	/// - `data`: metadata
	/// - `level`: upgrade level
	/// - `fee`: upgrade fee
	fn set_upgrade(
		who: AccountId,
		collection_id: CollectionId,
		item_id: ItemId,
		data: Metadata<StringLimit>,
		level: Level,
		fee: Balance,
	) -> DispatchResult;

	/// Upgrade
	///
	/// Upgrade item to the next level
	///
	/// Parameters:
	/// - `who`: who
	/// - `collection_id`: collection id
	/// - `item_id`: item id
	/// - `maybe_amount`: amount of items
	fn upgrade(
		who: AccountId,
		collection_id: CollectionId,
		item_id: ItemId,
		maybe_amount: Option<Amount>,
	) -> DispatchResult;
}

pub trait Transfer {
	fn transfer() -> DispatchResult;

	fn swap() -> DispatchResult;
}

pub trait Destroy<E> {
	fn destroy() -> Result<(), E>;
}

// pub trait Support {
// 	fn gen_id() -> CollectionId;
// }
