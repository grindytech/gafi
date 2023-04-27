use crate::common::{BlockNumber, ID};
use frame_support::{pallet_prelude::DispatchResult, BoundedVec};
use sp_runtime::{Percent, TokenError};
use sp_std::vec::Vec;

pub type Amount = u32;
pub type Level = u8;
pub type Metadata<S> = BoundedVec<u8, S>;

pub trait GameSetting<AccountId, GameId> {
	/// Do create a new game
	///
	/// Implementing the function create game
	///
	/// Parameters:
	/// - `id`: new game id
	/// - `owner`: owner
	/// - `maybe_admin`: admin
	/// - `maybe_name`: name
	///
	/// Weight: `O(1)`
	fn do_create_game(
		game_id: GameId,
		owner: AccountId,
		maybe_admin: Option<AccountId>,
		maybe_name: Option<Vec<u8>>,
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
		owner: AccountId,
		fee: Percent,
		start_block: BlockNumber,
	) -> DispatchResult;
}

pub trait Create<AccountId, GameId, CollectionId, ItemId> {
	/// Create game collection
	///
	/// Create collection for specific game
	///
	/// Parameters:
	/// - `game_id`: game id
	/// - `collection_id`: collection id
	/// - `owner`: owner
	/// - `admin`: admin
	fn create_game_collection(
		game_id: GameId,
		collection_id: CollectionId,
		owner: AccountId,
		admin: AccountId,
	) -> DispatchResult;

	/// Create collection
	///
	/// Create a pure collection
	///
	/// Parameters:
	/// - `collection_id`: collection id
	/// - `owner`: owner
	/// - `admin`: admin
	fn create_collection(
		collection_id: CollectionId,
		owner: AccountId,
		admin: AccountId,
	) -> DispatchResult;

	/// Create item
	///
	/// Create items for collection
	///
	/// Parameters:
	/// - `collection_id`: collection id
	/// - `item_id`: item id
	/// - `amount`: amount
	fn create_item(collection_id: CollectionId, item_id: ItemId, amount: Amount) -> DispatchResult;

	/// Add item
	///
	/// Add number amount of item in collection
	///
	/// Parameters:
	/// - `collection_id`: collection id
	/// - `item_id`: item id
	/// - `amount`: amount
	fn add_item(collection_id: CollectionId, item_id: ItemId, amount: Amount) -> DispatchResult;
}

pub trait Mutable<AccountId, GameId, CollectionId, ItemId> {
	/// Mint
	///
	/// Random mint item in the collection
	///
	/// Parameters:
	/// - `_who`: sender
	/// - `_collection_id`: collection id
	/// - `_maybe_target`: recipient account, default `minter`
	/// - `_maybe_amount`: amount of items to mint, default `1`
	///
	/// By default, this is not a supported operation.
	fn mint(
		_who: AccountId,
		_collection_id: CollectionId,
		_maybe_target: Option<AccountId>,
		_maybe_amount: Option<Amount>,
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
	fn burn(
		who: AccountId,
		collection_id: CollectionId,
		item_id: ItemId,
		maybe_amount: Option<Amount>,
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

pub trait Support {
	fn gen_id() -> Option<ID>;
}