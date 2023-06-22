use super::{Bundle, LootTable, Package, TradeType, MintSettings};
use frame_support::pallet_prelude::DispatchResult;

pub type Amount = u32;
pub type Level = u32;
pub trait GameSetting<AccountId, GameId> {
	/// Do create a new game
	///
	/// Implementing the function create game
	///
	/// Parameters:
	/// - `game`: new game id
	/// - `who`: signer and game owner
	/// - `admin`: admin
	fn do_create_game(game: &GameId, who: &AccountId, admin: &AccountId) -> DispatchResult;
}

pub trait MutateCollection<AccountId, GameId, CollectionId, CollectionConfig, Fee> {
	/// Do create game collection
	///
	/// The game admin creates a collection.
	/// Game and collection have the same owner and admin.
	///
	/// Parameters:
	/// - `who`: signer and game owner
	/// - `game`: game id
	/// - `config`: collection configuration
	fn do_create_game_collection(who: &AccountId, game: &GameId) -> DispatchResult;

	/// Do create collection
	///
	/// Create a pure collection
	///
	/// Parameters:
	/// - `who`: signer and collection owner
	/// - `admin`: admin role
	/// - `config`: collection configuration
	fn do_create_collection(who: &AccountId, admin: &AccountId) -> DispatchResult;

	/// Do Set Accept Adding
	///
	/// Accept to add collections to the game
	///
	/// - `who`: collection admin must be signed
	/// - `game`: game id
	/// - `ollection`: collection id to add to the game.
	fn do_set_accept_adding(
		who: &AccountId,
		game: &GameId,
		collection: &CollectionId,
	) -> DispatchResult;

	/// Do add collection
	///
	/// Add more amount on an existing game
	///
	/// Parameters:
	/// - `who`: signer and collection owner
	/// - `game`: game id
	/// - `collection`: collection id
	fn do_add_collection(
		who: &AccountId,
		game: &GameId,
		collection: &CollectionId,
	) -> DispatchResult;

	/// Do remove collection
	///
	/// Remove a colleciton from a game
	///
	/// - `who`: signer and collection owner
	/// - `game`: game id
	/// - `collection`: collection id
	fn do_remove_collection(
		who: &AccountId,
		game: &GameId,
		collection: &CollectionId,
	) -> DispatchResult;
}

pub trait CreateItem<AccountId, CollectionId, ItemId, ItemConfig> {
	/// Do Create item
	///
	/// Create items for collection
	///
	/// Parameters:
	/// - `who`: signer
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `config`: item config
	/// - `maybe_supply`: maximum number of item, None indicate an infinite supply
	fn do_create_item(
		who: &AccountId,
		collection: &CollectionId,
		item: &ItemId,
		config: &ItemConfig,
		maybe_supply: Option<u32>,
	) -> DispatchResult;

	/// Do add item
	///
	/// Add number amount of item in collection
	///
	/// Parameters:
	/// - `who`: signer
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `amount`: amount
	fn do_add_supply(
		who: &AccountId,
		collection: &CollectionId,
		item: &ItemId,
		amount: Amount,
	) -> DispatchResult;
}

///Trait to provide an interface for NFTs mining
pub trait Mining<AccountId, Price, CollectionId, ItemId, PoolId, BlockNumber> {

	/// Do create dynamic pool
	/// 
	/// Create a dynamic pool where the weight of the table changes after each loot.
	/// 
	/// - `pool`: mining pool id
	/// - `who`: signer and owner
	/// - `loot_table`: loot table
	/// - `fee`: mining fee
	/// - `admin`: admin
	fn do_create_dynamic_pool(
		pool: &PoolId,
		who: &AccountId,
		loot_table: LootTable<CollectionId, ItemId>,
		admin: &AccountId,
		mint_settings: MintSettings<Price, BlockNumber, CollectionId>,
	) -> DispatchResult;

	/// Do create dynamic pool
	/// 
	/// Create a stable pool where the weight of the table remains constant.
	/// 
	/// - `pool`: mining pool id
	/// - `who`: signer and owner
	/// - `loot_table`: loot table
	/// - `fee`: mining fee
	/// - `admin`: admin
	fn do_create_stable_pool(
		pool: &PoolId,
		who: &AccountId,
		loot_table: LootTable<CollectionId, ItemId>,
		admin: &AccountId,
		mint_settings: MintSettings<Price, BlockNumber, CollectionId>,
	) -> DispatchResult;

	/// Do mint dynamic pool
	/// 
	/// Do an `amount` of mining in a dynamic pool.
	/// 
	/// - `pool`: mining pool id
	/// - `who`: signer
	/// - `target`:  recipient account
	/// - `amount`: amount of item
	fn do_mint_dynamic_pool(
		pool: &PoolId,
		who: &AccountId,
		target: &AccountId,
		amount: Amount,
	) -> DispatchResult;

	/// Do mint dynamic pool
	/// 
	/// Do an `amount` of mining in a stable pool.
	/// 
	/// - `pool`: mining pool id
	/// - `who`: signer
	/// - `target`:  recipient account
	/// - `amount`: amount of item
	fn do_mint_stable_pool(
		pool: &PoolId,
		who: &AccountId,
		target: &AccountId,
		amount: Amount,
	) -> DispatchResult;

	/// Do Mint
	///
	/// Random mint item in a pool
	///
	/// Parameters:
	/// - `pool`: mining pool id
	/// - `who`: sender
	/// - `target`: recipient account
	/// - `amount`: amount of item
	fn do_mint(
		pool: &PoolId,
		who: &AccountId,
		target: &AccountId,
		amount: Amount,
	) -> DispatchResult;
}

pub trait MutateItem<AccountId, GameId, CollectionId, ItemId> {
	/// Burn
	///
	/// Burn item
	///
	/// Parameters:
	/// - `who`: item owner
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `amount`: amount of items to burn
	fn do_burn(
		who: &AccountId,
		collection: &CollectionId,
		item: &ItemId,
		amount: Amount,
	) -> DispatchResult;
}

pub trait UpgradeItem<AccountId, Balance, CollectionId, ItemId, ItemConfig, StringLimit> {
	/// Do Set Upgrade Item
	///
	/// Set upgrade item                          
	///
	/// Parameters:
	/// - `who`: item owner
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `data`: metadata
	/// - `level`: upgrade level
	/// - `fee`: upgrade fee
	fn do_set_upgrade_item(
		who: &AccountId,
		collection: &CollectionId,
		item: &ItemId,
		new_item: &ItemId,
		config: &ItemConfig,
		level: Level,
		fee: Balance,
	) -> DispatchResult;

	/// Do Upgrade Item
	///
	/// Upgrade item to the next level
	///
	/// Parameters:
	/// - `who`: who
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `amount`: amount of items
	fn do_upgrade_item(
		who: &AccountId,
		collection: &CollectionId,
		item: &ItemId,
		amount: Amount,
	) -> DispatchResult;
}

pub trait TransferItem<AccountId, CollectionId, ItemId> {
	/// Do Transfer Item
	///
	/// Transfer amount of item from `who` to `distination`
	///
	/// Parameters:
	/// - `who`: from account
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `destination`: destination account
	/// - `amount`: amount of items
	fn do_transfer_item(
		who: &AccountId,
		collection: &CollectionId,
		item: &ItemId,
		destination: &AccountId,
		amount: Amount,
	) -> DispatchResult;
}

pub trait Trade<AccountId, TradeId> {
	/// Do Cancel Trade
	///
	/// Cancel for any trade.
	///
	/// - `trade`: trade id
	/// - `who`: owner
	fn do_cancel_trade(trade: &TradeId, who: &AccountId, trade_type: TradeType) -> DispatchResult;
}

pub trait Retail<AccountId, CollectionId, ItemId, TradeId, Price, BlockNumber> {
	/// Do Set Price
	///
	/// Set item price for selling
	///
	/// Parameters:
	/// - `who`: seller
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `price`: price of a item
	fn do_set_price(
		trade: &TradeId,
		who: &AccountId,
		package: Package<CollectionId, ItemId>,
		price: Price,
		start_block: Option<BlockNumber>,
		end_block: Option<BlockNumber>,
	) -> DispatchResult;

	/// Do Buy Item
	///
	/// Buy items from specific seller
	///
	/// Parameters:
	/// - `who`: buyer
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `seller`: seller
	/// - `amount`: amount
	/// - `bid_price`: price of each item
	fn do_buy_item(
		trade: &TradeId,
		who: &AccountId,
		amount: Amount,
		bid_price: Price,
	) -> DispatchResult;

	/// Do Set Price
	///
	/// Set item price for selling
	///
	/// Parameters:
	/// - `who`: seller
	/// - `collection`: collection id
	/// - `item`: item id
	fn do_add_retail_supply(
		trade: &TradeId,
		who: &AccountId,
		supply: Package<CollectionId, ItemId>,
	) -> DispatchResult;

	/// Do Cancel Price
	///
	/// Cancel the trade, unlock the locked items, and unreserve the deposit.
	///
	/// Parameters:
	/// - `trade`: trade id
	/// - `who`: owner
	fn do_cancel_price(trade: &TradeId, who: &AccountId) -> DispatchResult;

	/// Do Set Buy
	///
	/// Set item want to buy.
	///
	/// Parameters:
	/// - `trade`: trade id
	/// - `who`: who
	/// - `package`: item want to buy
	/// - `price`: price of each
	fn do_set_buy(
		trade: &TradeId,
		who: &AccountId,
		package: Package<CollectionId, ItemId>,
		price: Price,
		start_block: Option<BlockNumber>,
		end_block: Option<BlockNumber>,
	) -> DispatchResult;

	/// Do Claim Set Buy
	///
	/// Sell item to buyer
	///
	/// Parameters:
	/// - `trade`: trade id
	/// - `who`: who
	/// - `amount`: amount item to sell
	/// - `bid_price`: bid_price of each
	fn do_claim_set_buy(
		trade: &TradeId,
		who: &AccountId,
		amount: Amount,
		ask_price: Price,
	) -> DispatchResult;

	/// Do Cancel Set Buy
	///
	/// Cancel set buy, unreserve deposit
	///
	/// Parameters:
	/// - `trade`: trade id
	/// - `who`: who
	fn do_cancel_set_buy(trade: &TradeId, who: &AccountId) -> DispatchResult;
}

pub trait Wholesale<AccountId, CollectionId, ItemId, TradeId, Price, BlockNumber> {
	/// Do Set Bundle
	///
	/// Bundling for sale
	///
	/// Parameters:
	/// - `trade`: trade id
	/// - `who`: seller
	/// - `bundle`: bundle
	/// - `price`: price of bundle
	fn do_set_bundle(
		trade: &TradeId,
		who: &AccountId,
		bundle: Bundle<CollectionId, ItemId>,
		price: Price,
		start_block: Option<BlockNumber>,
		end_block: Option<BlockNumber>,
	) -> DispatchResult;

	/// Do Buy Bundle
	///
	/// Buy a bundle from bundle id
	///
	/// Parameters:
	/// - `trade`: trade id
	/// - `who`: buyer
	/// - `bid_price`: the bid price for the bundle
	fn do_buy_bundle(trade: &TradeId, who: &AccountId, bid_price: Price) -> DispatchResult;

	/// Do Cancel Bundle
	///
	/// Cancel the bundle sale, unlock items, and unreserve the deposit.
	/// - `trade`: trade id
	/// - `who`: owner
	fn do_cancel_bundle(trade: &TradeId, who: &AccountId) -> DispatchResult;
}

/// Trait for wishlist functionality
pub trait Wishlist<AccountId, CollectionId, ItemId, TradeId, Price, BlockNumber> {
	/// Do Set Wishlist
	///
	/// Set a wishlist with the price
	///
	/// - `trade`: trade id
	/// - `who`: buyer
	/// - `bundle`: wishlist
	/// - `price`: price
	fn do_set_wishlist(
		trade: &TradeId,
		who: &AccountId,
		bundle: Bundle<CollectionId, ItemId>,
		price: Price,
		start_block: Option<BlockNumber>,
		end_block: Option<BlockNumber>,
	) -> DispatchResult;

	/// Do Set Wishlist
	///
	/// Fill the wishlist with the asking price.
	/// The asking price must be no greater than the wishlist price.
	///
	/// - `trade`: trade id
	/// - `who`: seller
	/// - `ask_price`: ask price
	fn do_claim_wishlist(trade: &TradeId, who: &AccountId, ask_price: Price) -> DispatchResult;

	/// Do Cancel Wishlist
	///
	/// Cancel the wishlist
	/// - `trade`: wishlist id
	/// - `who`: who
	fn do_cancel_wishlist(trade: &TradeId, who: &AccountId) -> DispatchResult;
}

/// Trait for swap items
pub trait Swap<AccountId, CollectionId, ItemId, TradeId, Price, BlockNumber> {
	/// Do Set Swap
	///
	/// Set a swap from a source bundle for a required bundle, maybe with price
	///
	/// - `trade`: trade id
	/// - `who`: who
	/// - `source`: bundle in
	/// - `required`: bundle out
	/// - `maybe_price`: maybe price required
	fn do_set_swap(
		trade: &TradeId,
		who: &AccountId,
		source: Bundle<CollectionId, ItemId>,
		required: Bundle<CollectionId, ItemId>,
		maybe_price: Option<Price>,
		start_block: Option<BlockNumber>,
		end_block: Option<BlockNumber>,
	) -> DispatchResult;

	/// Do Claim Swap
	///
	/// Make a swap with maybe bid price
	///
	/// - `trade`: trade id
	/// - `who`: who
	/// - `maybe_bid_price`: maybe bid price
	fn do_claim_swap(
		trade: &TradeId,
		who: &AccountId,
		maybe_bid_price: Option<Price>,
	) -> DispatchResult;

	/// Do Cancel Swap
	///
	/// Cancel swap
	/// - `trade`: swap id
	/// - `who`: who
	fn do_cancel_swap(trade: &TradeId, who: &AccountId) -> DispatchResult;
}

/// Trait for auction items
pub trait Auction<AccountId, CollectionId, ItemId, TradeId, Price, Block> {
	/// Do Set Auction
	///
	/// Set auction for a bundle may with minimum bid `maybe_price`.
	/// The one that has the highest bid when the auction ended wins.
	///
	/// - `trade`: auction id
	/// - `who`: who
	/// - `bundle`: bundle for auction
	/// - `maybe_price`: maybe minimum bid
	/// - `start_block`: the block when auction start
	/// - `duration`: duration
	fn do_set_auction(
		trade: &TradeId,
		who: &AccountId,
		bundle: Bundle<CollectionId, ItemId>,
		maybe_price: Option<Price>,
		start_block: Block,
		duration: Block,
	) -> DispatchResult;

	/// Do Bid Auction
	///
	/// Make a bid with price, the price must be higher than all bids before.
	///
	/// - `trade`: auction id
	/// - `who`: who
	/// - `price`: price
	fn do_bid_auction(trade: &TradeId, who: &AccountId, price: Price) -> DispatchResult;

	/// Do Claim Auction
	///
	/// Trigger end auction, any account can call.
	///
	/// - `trade`: auction id
	fn do_claim_auction(trade: &TradeId) -> DispatchResult;

	// fn do_set_candle_auction(
	// 	trade: &TradeId,
	// 	who: &AccountId,
	// 	bundle: Bundle<CollectionId, ItemId>,
	// 	maybe_price: Option<Price>,
	// 	start_block: Block,
	// 	early_end: Block,
	// 	end_block: Block,
	// ) -> DispatchResult;
}

pub trait Destroy<E> {
	fn destroy() -> Result<(), E>;
}
