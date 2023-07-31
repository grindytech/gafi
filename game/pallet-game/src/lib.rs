// This file is part of Gafi Network.

// Copyright (C) 2021-2022 Grindy Technologies.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Game Module
//!
//! A simple, secure module for dealing with in-game finances.

#![recursion_limit = "256"]
// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod features;
mod trades;
mod types;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;
use crate::weights::WeightInfo;
pub use weights::*;

use frame_support::{
	ensure,
	traits::{
		tokens::nonfungibles_v2::{Create, Inspect, InspectRole, Mutate, Transfer},
		Currency, Randomness, ReservableCurrency,
	},
	PalletId,
};
use frame_system::{
	offchain::{CreateSignedTransaction, SubmitTransaction},
	Config as SystemConfig,
};
use gafi_support::game::{
	Auction, CreateItem, GameSetting, Level, LootTable, Mining, MutateCollection, MutateItem,
	Package, Retail, Swap, Trade, TradeType, TransferItem, UpgradeItem, Wholesale, Wishlist,
};
use pallet_nfts::{
	AttributeNamespace, CollectionConfig, Incrementable, ItemConfig, WeightInfo as NftsWeightInfo,
};
use sp_core::offchain::KeyTypeId;
use sp_runtime::traits::{StaticLookup, TrailingZeroInput};
use sp_std::vec::Vec;
use types::*;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"gafi");
pub const UNSIGNED_TXS_PRIORITY: u64 = 10;

pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);
	pub struct TestAuthId;

	// implemented for runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::{OptionQuery, ValueQuery, *},
		traits::tokens::nonfungibles_v2::InspectRole,
		Blake2_128Concat, Twox64Concat,
	};
	use sp_core::Get;

	use super::*;
	use frame_system::pallet_prelude::{OriginFor, *};
	use gafi_support::game::{Bundle, Loot, NFT};
	use pallet_nfts::CollectionRoles;

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(_);

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<GameId, TradeId, BlockNumber, PoolId> {
		fn game(i: u16) -> GameId;

		fn trade(i: u16) -> TradeId;

		fn block(i: u16) -> BlockNumber;

		fn pool(i: u16) -> PoolId;
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<GameId: From<u16>, TradeId: From<u16>, BlockNumber: From<u16>, PoolId: From<u16>>
		BenchmarkHelper<GameId, TradeId, BlockNumber, PoolId> for ()
	{
		fn game(i: u16) -> GameId {
			i.into()
		}

		fn trade(i: u16) -> TradeId {
			i.into()
		}

		fn block(i: u16) -> BlockNumber {
			i.into()
		}

		fn pool(i: u16) -> PoolId {
			i.into()
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config<I: 'static = ()>:
		frame_system::Config + pallet_nfts::Config + CreateSignedTransaction<Call<Self, I>>
	{
		/// The Game's pallet id
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Weight information for pallet-nfts.
		type NftsWeightInfo: NftsWeightInfo;

		/// The currency mechanism, used for paying for reserves.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// pallet_nfts
		type Nfts: Mutate<Self::AccountId, ItemConfig>
			+ Transfer<Self::AccountId>
			+ Create<
				Self::AccountId,
				CollectionConfig<BalanceOf<Self, I>, Self::BlockNumber, Self::CollectionId>,
			> + Inspect<Self::AccountId>
			+ Inspect<Self::AccountId, ItemId = Self::ItemId, CollectionId = Self::CollectionId>
			+ InspectRole<Self::AccountId>;

		/// generate random ID
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// The type used to identify a unique game
		type GameId: Member + Parameter + MaxEncodedLen + Copy + Incrementable;

		/// The type used to identify a unique trade
		type TradeId: Member + Parameter + MaxEncodedLen + Copy + Incrementable;

		/// The type used to identify a unique mining pool
		type PoolId: Member + Parameter + MaxEncodedLen + Copy + Incrementable;

		/// The basic amount of funds that must be reserved for a game.
		#[pallet::constant]
		type GameDeposit: Get<BalanceOf<Self, I>>;

		/// The basic amount of funds that must be reserved for a mining pool.
		#[pallet::constant]
		type MiningPoolDeposit: Get<BalanceOf<Self, I>>;

		/// Maximum number of collections in a  game.
		#[pallet::constant]
		type MaxGameCollection: Get<u32>;

		/// Maximum number of games a collection can share.
		#[pallet::constant]
		type MaxGameShare: Get<u32>;

		/// Maximum number of item that a collection could has
		#[pallet::constant]
		type MaxItem: Get<u32>;

		/// Maximum number of item minted once
		#[pallet::constant]
		type MaxMintItem: Get<u32>;

		/// The basic amount of funds that must be reserved for any upgrade.
		#[pallet::constant]
		type UpgradeDeposit: Get<BalanceOf<Self, I>>;

		/// Maximum collection in a bundle for trade
		#[pallet::constant]
		type MaxBundle: Get<u32>;

		/// Maximum number of loot that a table could has
		#[pallet::constant]
		type MaxLoot: Get<u32>;

		/// The basic amount of funds that must be reserved for any bundle.
		#[pallet::constant]
		type BundleDeposit: Get<BalanceOf<Self, I>>;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type Helper: BenchmarkHelper<Self::GameId, Self::TradeId, Self::BlockNumber, Self::PoolId>;
	}

	/// Storing basic game info
	#[pallet::storage]
	pub(super) type Game<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::GameId, GameDetailsFor<T, I>>;

	/// The games owned by any given account; set out this way so that games owned by
	/// a single account can be enumerated.
	#[pallet::storage]
	pub type GameAccount<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::GameId,
		(),
		OptionQuery,
	>;

	/// Storing next game id
	#[pallet::storage]
	pub(super) type NextGameId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::GameId, OptionQuery>;

	/// Storing next trade id
	#[pallet::storage]
	pub(super) type NextTradeId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::TradeId, OptionQuery>;

	/// Storing next mining pool id
	#[pallet::storage]
	pub(super) type NextPoolId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::PoolId, OptionQuery>;

	/// Collections in the game
	#[pallet::storage]
	pub(super) type CollectionsOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		T::GameId,
		BoundedVec<T::CollectionId, T::MaxGameCollection>,
		ValueQuery,
	>;

	/// Collection belongs to
	#[pallet::storage]
	pub(super) type GamesOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		T::CollectionId,
		BoundedVec<T::GameId, T::MaxGameShare>,
		ValueQuery,
	>;

	/// Game roles
	#[pallet::storage]
	pub(super) type GameRoleOf<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::GameId,
		Blake2_128Concat,
		T::AccountId,
		CollectionRoles,
		OptionQuery,
	>;

	/// Item balances of account
	#[pallet::storage]
	pub(super) type ItemBalanceOf<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, T::CollectionId>,
			NMapKey<Twox64Concat, T::ItemId>,
		),
		u32,
		ValueQuery,
	>;

	/// Storing reserved balance
	#[pallet::storage]
	pub(super) type ReservedBalanceOf<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, T::CollectionId>,
			NMapKey<Twox64Concat, T::ItemId>,
		),
		u32,
		ValueQuery,
	>;

	/// Storing Nft supplies, `None` indicates infinite supply
	#[pallet::storage]
	pub(super) type SupplyOf<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::CollectionId,
		Blake2_128Concat,
		T::ItemId,
		Option<u32>,
		OptionQuery,
	>;

	/// Item reserve for random minting created by the owner
	#[pallet::storage]
	pub(super) type LootTableOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128,
		T::PoolId,
		BoundedVec<Loot<T::CollectionId, T::ItemId>, T::MaxLoot>,
		ValueQuery,
	>;

	/// Storing mining pool configuration
	#[pallet::storage]
	pub(super) type PoolOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::PoolId, PoolDetailsFor<T, I>, OptionQuery>;

	/// Level of item
	#[pallet::storage]
	pub(super) type LevelOf<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::CollectionId,
		Blake2_128Concat,
		T::ItemId,
		Level,
		ValueQuery,
	>;

	/// Storing the original items of the upgraded items
	#[pallet::storage]
	pub(super) type OriginItemOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		(T::CollectionId, T::ItemId),
		(T::CollectionId, T::ItemId),
		OptionQuery,
	>;

	/// Storing the upgrade config
	#[pallet::storage]
	pub(super) type UpgradeConfigOf<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::CollectionId>,
			NMapKey<Blake2_128Concat, T::ItemId>, // original item
			NMapKey<Blake2_128Concat, Level>,     // level upgrade
		),
		ItemUpgradeConfigFor<T, I>,
		OptionQuery,
	>;

	/// Storing random seed generated from the off-chain worker every block
	#[pallet::storage]
	pub(crate) type RandomSeed<T: Config<I>, I: 'static = ()> =
		StorageValue<_, [u8; 32], ValueQuery>;

	/// Storing bundle
	#[pallet::storage]
	pub(super) type BundleOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::TradeId, BundleFor<T, I>, ValueQuery>;

	/// Storing trade configuration
	#[pallet::storage]
	pub(super) type TradeConfigOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::TradeId, TradeConfigFor<T, I>, OptionQuery>;

	/// Storing auction configuration
	#[pallet::storage]
	pub(super) type AuctionConfigOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::TradeId,
		AuctionConfig<T::AccountId, BalanceOf<T, I>, T::BlockNumber>,
		OptionQuery,
	>;

	/// Storing the highest bid of auction
	#[pallet::storage]
	pub(super) type HighestBidOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::TradeId, (T::AccountId, BalanceOf<T, I>), OptionQuery>;

	/// Store accepts to add collections to the games
	#[pallet::storage]
	pub(super) type AddingAcceptance<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::CollectionId, T::GameId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		GameCreated {
			who: T::AccountId,
			game: T::GameId,
		},
		CollectionCreated {
			who: T::AccountId,
			collection: T::CollectionId,
		},
		AddingAcceptanceSet {
			who: T::AccountId,
			game: T::GameId,
			collection: T::CollectionId,
		},
		CollectionAdded {
			who: T::AccountId,
			game: T::GameId,
			collection: T::CollectionId,
		},
		ItemCreated {
			who: T::AccountId,
			collection: T::CollectionId,
			item: T::ItemId,
			maybe_supply: Option<u32>,
		},
		ItemAdded {
			who: T::AccountId,
			collection: T::CollectionId,
			item: T::ItemId,
			amount: u32,
		},
		Minted {
			pool: T::PoolId,
			who: T::AccountId,
			target: T::AccountId,
			nfts: Vec<NFT<T::CollectionId, T::ItemId>>,
		},
		Burned {
			who: T::AccountId,
			collection: T::CollectionId,
			item: T::ItemId,
			amount: u32,
		},
		Transferred {
			from: T::AccountId,
			collection: T::CollectionId,
			item: T::ItemId,
			dest: T::AccountId,
			amount: u32,
		},
		UpgradeSet {
			who: T::AccountId,
			collection: T::CollectionId,
			item: T::ItemId,
			new_item: T::ItemId,
			level: Level,
		},
		Upgraded {
			who: T::AccountId,
			collection: T::CollectionId,
			item: T::ItemId,
			new_item: T::ItemId,
			amount: u32,
		},
		PriceSet {
			trade: T::TradeId,
			who: T::AccountId,
			collection: T::CollectionId,
			item: T::ItemId,
			amount: u32,
			unit_price: BalanceOf<T, I>,
		},
		ItemBought {
			trade: T::TradeId,
			who: T::AccountId,
			amount: u32,
			bid_unit_price: BalanceOf<T, I>,
		},
		BundleSet {
			trade: T::TradeId,
			who: T::AccountId,
			bundle: Bundle<T::CollectionId, T::ItemId>,
			price: BalanceOf<T, I>,
		},
		BundleBought {
			trade: T::TradeId,
			who: T::AccountId,
			bid_price: BalanceOf<T, I>,
		},
		TradeCanceled {
			trade: T::TradeId,
			who: T::AccountId,
		},
		WishlistSet {
			trade: T::TradeId,
			who: T::AccountId,
			wishlist: Bundle<T::CollectionId, T::ItemId>,
			price: BalanceOf<T, I>,
		},
		WishlistFilled {
			trade: T::TradeId,
			who: T::AccountId,
			ask_price: BalanceOf<T, I>,
		},
		CollectionRemoved {
			who: T::AccountId,
			game: T::GameId,
			collection: T::CollectionId,
		},
		SwapSet {
			trade: T::TradeId,
			who: T::AccountId,
			source: Bundle<T::CollectionId, T::ItemId>,
			required: Bundle<T::CollectionId, T::ItemId>,
			maybe_price: Option<BalanceOf<T, I>>,
		},
		SwapClaimed {
			trade: T::TradeId,
			who: T::AccountId,
			maybe_bid_price: Option<BalanceOf<T, I>>,
		},
		AuctionSet {
			trade: T::TradeId,
			who: T::AccountId,
			source: Bundle<T::CollectionId, T::ItemId>,
			maybe_price: Option<BalanceOf<T, I>>,
			start_block: T::BlockNumber,
			duration: T::BlockNumber,
		},
		Bid {
			trade: T::TradeId,
			who: T::AccountId,
			bid: BalanceOf<T, I>,
		},
		AuctionClaimed {
			trade: T::TradeId,
			maybe_bid: Option<(T::AccountId, BalanceOf<T, I>)>,
		},
		BuySet {
			trade: T::TradeId,
			who: T::AccountId,
			collection: T::CollectionId,
			item: T::ItemId,
			amount: u32,
			unit_price: BalanceOf<T, I>,
		},
		SetBuyClaimed {
			trade: T::TradeId,
			who: T::AccountId,
			amount: u32,
			ask_unit_price: BalanceOf<T, I>,
		},
		MiningPoolCreated {
			pool: T::PoolId,
			who: T::AccountId,
			pool_type: PoolType,
			table: LootTable<T::CollectionId, T::ItemId>,
		},
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		NoPermission,

		UnknownGame,
		UnknownCollection,
		UnknownItem,
		UnknownTrade,
		UnknownUpgrade,
		UnknownAuction,
		UnknownBid,
		UnknownAcceptance,
		UnknownMiningPool,

		/// Exceed the maximum allowed item in a collection
		ExceedMaxItem,
		/// The number minted items require exceeds the available items in the reserve
		ExceedTotalAmount,
		/// The number minted items require exceeds the amount allowed per tx
		ExceedAllowedAmount,
		/// Exceed the maximum allowed collection in a game
		ExceedMaxCollection,
		/// Exceeded the maximum number of games that can be shared between collections
		ExceedMaxGameShare,
		/// Exceed max collections in a bundle
		ExceedMaxBundle,
		/// Exceed max loots in a table
		ExceedMaxLoot,

		SoldOut,
		/// Too many attempts
		WithdrawReserveFailed,
		UpgradeExists,
		/// Add the same collection into a game
		CollectionExists,

		InsufficientItemBalance,
		InsufficientReservedBalance,

		/// item amount = 0
		InvalidAmount,

		/// Transfer is locked for any trade
		ItemLocked,

		/// The bid is lower than the set price.
		BidTooLow,

		/// The asking price is higher than the set price.
		AskTooHigh,

		// Id in use
		GameIdInUse,
		TradeIdInUse,
		PoolIdInUse,

		// trade
		TradeNotStarted,
		TradeEnded,
		// Retail trade
		IncorrectCollection,
		IncorrectItem,

		// auction
		AuctionInProgress,
		AuctionNotStarted,
		AuctionEnded,

		// trade type
		NotSetPrice,
		NotBundle,
		NotWishlist,
		NotSwap,
		NotAuction,
		NotSetBuy,

		//mining pool
		InfiniteSupply,
		NotInfiniteSupply,
		MintFailed,
		MintNotStarted,
		MintEnded,
		NotWhitelisted,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn offchain_worker(_block_number: BlockNumberFor<T>) {
			let _ = Self::submit_random_seed_raw_unsigned(_block_number);
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Create a new game.
		///
		/// Origin must be Signed.
		///
		/// If the origin is Signed, then funds of signer are reserved: `GameDeposit`.
		///
		/// - `admin`: the admin of the game.
		///
		/// Emits `GameCreated`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::create_game(1_u32))]
		pub fn create_game(origin: OriginFor<T>, admin: AccountIdLookupOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let admin = T::Lookup::lookup(admin)?;
			let game = Self::get_game_id();
			Self::do_create_game(&game, &sender, &admin)?;
			Ok(())
		}

		/// Create a collection in the game.
		///
		/// Origin must be Signed and the sender should be the Admin the the `game`.
		///
		/// If the origin is Signed, then funds of signer are reserved: `CollectionDeposit`.
		///
		/// - `game`: the game id.
		///
		/// Emits `CollectionCreated`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::create_game_collection(1_u32))]
		pub fn create_game_collection(origin: OriginFor<T>, game: T::GameId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_create_game_collection(&sender, &game)?;
			Ok(())
		}

		/// Create a new collection.
		///
		/// This new collection has no items initially and its owner is the origin.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// `CollectionDeposit` funds of sender are reserved.
		///
		/// Parameters:
		/// - `admin`: The admin of this collection. The admin is the initial address of each
		/// member of the collection's admin team.
		///
		/// Emits `CollectionCreated`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::create_collection(1_u32))]
		pub fn create_collection(
			origin: OriginFor<T>,
			admin: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let admin = T::Lookup::lookup(admin)?;
			let sender = ensure_signed(origin)?;
			Self::do_create_collection(&sender, &admin)?;
			Ok(())
		}

		/// Set acceptance of ownership for a particular account.
		///
		/// Origin must be `Signed` and the sender should be the Admin of `collection`.
		///
		/// - `game`: Game ID.
		/// - `collection`: Collection ID.
		///
		/// Emits `AddingAcceptanceSet`.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_accept_adding(1_u32))]
		pub fn set_accept_adding(
			origin: OriginFor<T>,
			game: T::GameId,
			collection: T::CollectionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_set_accept_adding(&sender, &game, &collection)?;
			Ok(())
		}

		/// Add a collection to the game.
		///
		/// The origin must be Signed and the sender should be the Admin of the `game`.
		///
		/// Parameters:
		/// - `game`: Game ID.
		/// - `collection`: Collection ID.
		///
		/// Emits `CollectionAdded`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::add_game_collection(1_u32))]
		pub fn add_game_collection(
			origin: OriginFor<T>,
			game: T::GameId,
			collection: T::CollectionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_add_collection(&sender, &game, &collection)?;
			Ok(())
		}

		/// Create an certain amount of item for a particular collection.
		///
		/// The origin must be Signed and the sender should be the Admin of `collection`.
		///
		/// - `collection`: The collection of the item to be minted.
		/// - `item`: An identifier of the new item.
		/// - `config`: Item Config.
		/// - `maybe_supply`: Item supply, None indicates the infinite supply.
		///
		/// Emits `ItemCreated` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::create_item(1_u32))]
		pub fn create_item(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
			config: ItemConfig,
			maybe_supply: Option<u32>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_create_item(&sender, &collection, &item, &config, maybe_supply)?;
			Ok(())
		}

		/// Add supplies for the item.
		///
		/// The origin must be Signed and the sender should be the Admin of `collection`.
		///
		/// - `collection`: The collection of the item to be minted.
		/// - `item`: An identifier of the new item.
		/// - `amount`: Supply amount.
		///
		/// Emits `ItemAdded` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::add_supply(1_u32))]
		pub fn add_supply(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
			amount: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_add_supply(&sender, &collection, &item, amount)?;
			Ok(())
		}

		/// Mint an amount of item on a particular mining pool.
		///
		/// The origin must be Signed and the sender must comply with the `mint_settings` rules.
		///
		/// - `pool`: The pool to be minted.
		/// - `mint_to`: Account into which the item will be minted.
		/// - `amount`: The amount may be minted.
		///
		/// Emits `Minted` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::mint(1_u32))]
		pub fn mint(
			origin: OriginFor<T>,
			pool: T::PoolId,
			mint_to: AccountIdLookupOf<T>,
			amount: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let target = T::Lookup::lookup(mint_to)?;
			Self::do_mint(&pool, &sender, &target, amount)?;
			Ok(())
		}

		/// Burn amount of item.
		///
		/// The origin must conform to `ForceOrigin` or must be Signed and the signing account must
		/// be the owner of the `item` and has sufficient item balance.
		///
		/// - `collection`: The collection of the item to be burned.
		/// - `item`: The item to be burned.
		/// - `amount`: The amount of item to be burned.
		///
		/// Emits `Burned`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(8)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::burn(1_u32))]
		pub fn burn(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
			amount: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_burn(&sender, &collection, &item, amount)?;
			Ok(())
		}

		/// Move an item from the sender account to another.
		///
		/// Origin must be Signed and the signing account must be the owner of the `item`.
		///
		/// Arguments:
		/// - `collection`: The collection of the item to be transferred.
		/// - `item`: The item to be transferred.
		/// - `dest`: The account to receive ownership of the item.
		/// - `amount`: The amount of item to be transferred.
		///
		/// Emits `Transferred`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(9)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::transfer(1_u32))]
		pub fn transfer(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
			dest: AccountIdLookupOf<T>,
			amount: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let destination = T::Lookup::lookup(dest)?;
			Self::do_transfer_item(&sender, &collection, &item, &destination, amount)?;
			Ok(())
		}

		// SBP-M2: As weights are 2D, we need to take care of proof_size as well, here we should
		// check the length of `Vec` and it should be returned with DispatchResultWithPostInfo
		// return type. This will help in calculating the actual `proof_size` used in the
		// transaction. For this, benchmark should also be updated in order to incorporate this
		// change.

		/// Set upgrade rule for item.
		///
		/// Origin must be Signed and signer should be the Admin of `collection`.
		///
		/// Arguments:
		/// - `collection`: The collection of the item to be upgrade-rule set.
		/// - `item`: The item to be upgrade-rule set.
		/// - `new_item`: An identifier of the new item.
		/// - `config`: Item config of `new_item`.
		/// - `data`: `new_item` metadata.
		/// - `level`: Upgrade level.
		/// - `fee`: Upgrade fee.
		///
		/// Emits `UpgradeSet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(10)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_upgrade_item(1_u32))]
		pub fn set_upgrade_item(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
			new_item: T::ItemId,
			config: ItemConfig,
			data: BoundedVec<u8, T::StringLimit>,
			level: Level,
			fee: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			pallet_nfts::pallet::Pallet::<T>::set_metadata(origin, collection, item, data)?;
			Self::do_set_upgrade_item(&sender, &collection, &item, &new_item, &config, level, fee)?;
			Ok(())
		}

		/// Upgrade certain number of items.
		///
		/// The origin must be signed and the signer must have a sufficient `amount` of `items`.
		///
		/// Signer must pay `fee` * `amount` to upgrade the item.
		///
		/// Arguments:
		/// - `collection`: The collection of the item to be upgraded.
		/// - `item`: The item to be upgraded.
		/// - `amount`: The amount of `item` to be upgraded.
		///
		/// Emits `Upgraded`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(11)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::upgrade_item(1_u32))]
		pub fn upgrade_item(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
			amount: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_upgrade_item(&sender, &collection, &item, amount)?;
			Ok(())
		}

		/// Submit random seed from offchain-worker to runtime.
		///
		/// Only called by offchain-worker.
		///
		/// Arguments:
		/// - `seed`: random seed value.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(12)]
		#[pallet::weight({0})]
		pub fn submit_random_seed_unsigned(origin: OriginFor<T>, seed: [u8; 32]) -> DispatchResult {
			ensure_none(origin)?;
			RandomSeed::<T, I>::set(seed);
			Ok(())
		}

		/// Set the price for a package.
		///
		/// Origin must be Signed and must be the owner of the `item`.
		///
		/// - `package`: a number of an item in a collection to set the price for.
		/// - `unit_price`: The price for each item.
		/// - `start_block`: The block to start setting the price.
		/// - `end_block`: The block to end setting the price.
		///
		/// Emits `PriceSet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(13)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_price(1_u32))]
		pub fn set_price(
			origin: OriginFor<T>,
			package: Package<T::CollectionId, T::ItemId>,
			unit_price: BalanceOf<T, I>,
			start_block: Option<T::BlockNumber>,
			end_block: Option<T::BlockNumber>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let trade = Self::get_trade_id();
			Self::do_set_price(&trade, &sender, package, unit_price, start_block, end_block)?;
			Ok(())
		}

		/// Buy certain number of items from `set_price`.
		///
		/// Origin must be Signed.
		///
		/// - `trade`: The set_price trade id.
		/// - `amount`: Number of items to buy.
		/// - `bid_price`: Bid for each item, `bid_price` must be equal to or higher than
		///   `price_unit`.
		///
		/// Emits `ItemBought`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(14)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::buy_item(1_u32))]
		pub fn buy_item(
			origin: OriginFor<T>,
			trade: T::TradeId,
			amount: u32,
			bid_price: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_buy_item(&trade, &sender, amount, bid_price)?;
			Ok(())
		}

		/// Add more items to set the price in `set_price`.
		///
		/// Origin must be Signed and must be the owner of the `trade`.
		///
		/// - `trade`: The set_price trade id.
		/// - `supply`: The number of items to be added.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(15)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::add_retail_supply(1_u32))]
		pub fn add_retail_supply(
			origin: OriginFor<T>,
			trade: T::TradeId,
			supply: Package<T::CollectionId, T::ItemId>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_add_retail_supply(&trade, &sender, supply)?;
			Ok(())
		}

		// SBP-M2: DispatchResultWithPostInfo should be used for actual `proof_size`.
		// Please refer set_upgrade_item's comment.

		/// Set the price for the `bundle`.
		///
		/// Origin must be Signed and must be the owner of the `bundle`.
		///
		/// - `bundle`: A group of items may be from different collections to set price for.
		/// - `price`: The price the `bundle`.
		/// - `start_block`: The block to start setting the price.
		/// - `end_block`: The block to end setting the price.
		///
		/// Emits `BundleSet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(16)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_bundle(1_u32))]
		pub fn set_bundle(
			origin: OriginFor<T>,
			bundle: Bundle<T::CollectionId, T::ItemId>,
			price: BalanceOf<T, I>,
			start_block: Option<T::BlockNumber>,
			end_block: Option<T::BlockNumber>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let trade = Self::get_trade_id();
			Self::do_set_bundle(&trade, &sender, bundle, price, start_block, end_block)?;
			Ok(())
		}

		/// Buy a bundle from `set_bundle`.
		///
		/// Origin must be Signed.
		///
		/// - `trade`: set_bundle trade id.
		/// - `bid_price`: The price the sender is willing to pay.
		///
		/// Emits `BundleSet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(17)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::buy_bundle(1_u32))]
		pub fn buy_bundle(
			origin: OriginFor<T>,
			trade: T::TradeId,
			bid_price: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_buy_bundle(&trade, &sender, bid_price)?;
			Ok(())
		}

		/// Cancel a trade in `trade_type` by id `trade`.
		///
		/// Origin must be Signed and signer must be the trade owner.
		///
		/// - `trade`: Trade id.
		/// - `trade_type`: Trade type.
		///
		/// Emits `TradeCanceled`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(18)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::cancel_trade(1_u32))]
		pub fn cancel_trade(
			origin: OriginFor<T>,
			trade: T::TradeId,
			trade_type: TradeType,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_cancel_trade(&trade, &sender, trade_type)?;
			Ok(())
		}

		// SBP-M2: DispatchResultWithPostInfo should be used for actual `proof_size`.
		// Please refer set_upgrade_item's comment.

		/// Set up a purchase for `bundle`.
		///
		/// Origin must be Signed.
		///
		/// - `bundle`:  A group of items may be from different collections want to buy.
		/// - `price`: The price the sender is willing to pay.
		/// 	- `start_block`: The block to start set wishlist.
		/// - `end_block`: The block to end set wishlist.
		///
		/// Emits `WishlistSet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(19)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_wishlist(1_u32))]
		pub fn set_wishlist(
			origin: OriginFor<T>,
			bundle: Bundle<T::CollectionId, T::ItemId>,
			price: BalanceOf<T, I>,
			start_block: Option<T::BlockNumber>,
			end_block: Option<T::BlockNumber>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let trade = Self::get_trade_id();
			Self::do_set_wishlist(&trade, &sender, bundle, price, start_block, end_block)?;
			Ok(())
		}

		/// Sell the bundle for `set_wishlist`.
		///
		/// Origin must be Signed.
		///
		/// - `trade`:  The set_wishlist trade id.
		/// - `ask_price`: The price the sender is willing to accept.
		///
		/// Emits `WishlistFilled`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(20)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::claim_wishlist(1_u32))]
		pub fn claim_wishlist(
			origin: OriginFor<T>,
			trade: T::TradeId,
			ask_price: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_claim_wishlist(&trade, &sender, ask_price)?;
			Ok(())
		}

		/// Remove a collection in the game.
		///
		/// Origin must be Signed and signer should be the Admin of the game or collection.
		///
		/// - `game`:  The game id.
		/// - `ask_price`: The collection id.
		///
		/// Emits `CollectionRemoved`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(21)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::remove_collection(1_u32))]
		pub fn remove_collection(
			origin: OriginFor<T>,
			game: T::GameId,
			collection: T::CollectionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_remove_collection(&sender, &game, &collection)?;
			Ok(())
		}

		/// Disallow further unprivileged transfer or trade of an item.
		/// Simply re-call `lock_item_transfer` of `pallet-nfts`.
		///
		/// Origin must be Signed and the sender should be the Freezer of the `collection`.
		///
		/// - `collection`: The collection of the item to be changed.
		/// - `item`: The item to become non-transferable.
		///
		/// Emits `ItemTransferLocked`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(22)]
		#[pallet::weight(T::NftsWeightInfo::lock_item_transfer())]
		pub fn lock_item_transfer(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
		) -> DispatchResult {
			pallet_nfts::pallet::Pallet::<T>::lock_item_transfer(origin, collection, item)
		}

		/// Re-allow unprivileged transfer of an item.
		/// Simply re-call `unlock_item_transfer` of `pallet-nfts`.
		///
		/// Origin must be Signed and the sender should be the Freezer of the `collection`.
		///
		/// - `collection`: The collection of the item to be changed.
		/// - `item`: The item to become transferable.
		///
		/// Emits `ItemTransferUnlocked`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(23)]
		#[pallet::weight(T::NftsWeightInfo::unlock_item_transfer())]
		pub fn unlock_item_transfer(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
		) -> DispatchResult {
			pallet_nfts::pallet::Pallet::<T>::unlock_item_transfer(origin, collection, item)
		}

		// SBP-M2: DispatchResultWithPostInfo should be used for actual `proof_size`.
		// Please refer set_upgrade_item's comment.

		/// Set a swap to exchange `source` to `required`.
		///
		/// Origin must be Signed and the sender must be the owner of `source`.
		///
		/// - `source`: Bundle in.
		/// - `required`: Bundle out.
		/// - `maybe_price`: Maybe the price that sender willing to accept.
		/// 	- `start_block`: The block to start set swap.
		/// - `end_block`: The block to end set swap.
		///
		/// Emits `SwapSet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(24)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_swap(1_u32))]
		pub fn set_swap(
			origin: OriginFor<T>,
			source: Bundle<T::CollectionId, T::ItemId>,
			required: Bundle<T::CollectionId, T::ItemId>,
			maybe_price: Option<BalanceOf<T, I>>,
			start_block: Option<T::BlockNumber>,
			end_block: Option<T::BlockNumber>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let trade = Self::get_trade_id();
			Self::do_set_swap(
				&trade,
				&sender,
				source,
				required,
				maybe_price,
				start_block,
				end_block,
			)?;
			Ok(())
		}

		/// Make an exchange for `set_swap`.
		///
		/// Origin must be Signed.
		///
		/// - `trade`: The set_swap trade id.
		/// - `maybe_bid_price`: Maybe a price sender willing to pay.
		///
		/// Emits `SwapClaimed`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(25)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::claim_swap(1_u32))]
		pub fn claim_swap(
			origin: OriginFor<T>,
			trade: T::TradeId,
			maybe_bid_price: Option<BalanceOf<T, I>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_claim_swap(&trade, &sender, maybe_bid_price)?;
			Ok(())
		}

		// SBP-M2: DispatchResultWithPostInfo should be used for actual `proof_size`.
		// Please refer set_upgrade_item's comment

		/// Create a auction for `source`.
		///
		/// Origin must be Signed and signer must be the owner of the `source`.
		/// The last bidder will win the auction.
		///
		/// - `source`: The bundle for auction.
		/// - `maybe_price`: Maybe a minimum bid.
		/// - `start_block`: The block to start the auction.
		/// - `duration`: The duration of the auction and measured by the number of blocks.
		///
		/// Emits `AuctionSet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(26)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_auction(1_u32))]
		pub fn set_auction(
			origin: OriginFor<T>,
			source: Bundle<T::CollectionId, T::ItemId>,
			maybe_price: Option<BalanceOf<T, I>>,
			start_block: T::BlockNumber,
			duration: T::BlockNumber,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let trade = Self::get_trade_id();
			Self::do_set_auction(&trade, &sender, source, maybe_price, start_block, duration)?;
			Ok(())
		}

		/// Make a bid for the auction.
		///
		/// Origin must be Signed.
		///
		/// - `trade`: The auction id.
		/// - `bid`: The bid, `bid` must be higher than the minimum bid and higher than the previous
		///   bid.
		///
		/// Emits `Bid`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(27)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::bid_auction(1_u32))]
		pub fn bid_auction(
			origin: OriginFor<T>,
			trade: T::TradeId,
			bid: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_bid_auction(&trade, &sender, bid)?;
			Ok(())
		}

		/// Handling an auction after it's over.
		///
		/// The last bidder will win the auction.
		/// If there is no bid, the NFT in the auction will be refunded.
		///
		/// Origin must be Signed.
		///
		/// - `trade`: The auction id.
		///
		/// Emits `AuctionClaimed`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(28)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::claim_auction(1_u32))]
		pub fn claim_auction(origin: OriginFor<T>, trade: T::TradeId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			Self::do_claim_auction(&trade)?;
			Ok(())
		}

		/// Set up a purchase for `package`.
		///
		/// It is possible to trade for a small part of the `package`.
		///
		/// Origin must be Signed.
		///
		/// - `package`: A number of an item in a collection want to buy.
		/// - `unit_price`: The price of each item the sender is willing to pay.
		/// - `start_block`: The block to start set buy.
		/// - `end_block`: The block to end set buy.
		///
		/// Emits `BuySet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(29)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_buy(1_u32))]
		pub fn set_buy(
			origin: OriginFor<T>,
			package: Package<T::CollectionId, T::ItemId>,
			unit_price: BalanceOf<T, I>,
			start_block: Option<T::BlockNumber>,
			end_block: Option<T::BlockNumber>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let trade = Self::get_trade_id();
			Self::do_set_buy(&trade, &sender, package, unit_price, start_block, end_block)?;
			Ok(())
		}

		/// Sell ​​`amount` of the item for `set_buy`.
		///
		/// Origin must be Signed.
		///
		/// - `trade`: The set_buy trade id.
		/// - `amount`: The amount of items to sell.
		/// - `ask_price`: The price that the sender willing to accept.
		///
		/// Emits `BuySet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(30)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::claim_set_buy(1_u32))]
		pub fn claim_set_buy(
			origin: OriginFor<T>,
			trade: T::TradeId,
			amount: u32,
			ask_price: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_claim_set_buy(&trade, &sender, amount, ask_price)?;
			Ok(())
		}

		/// Set an attribute for a collection or item.
		///
		/// Simply re-call `set_attribute` of `pallet-nfts`.
		///
		/// Origin must be Signed and must conform to the namespace ruleset:
		/// - `CollectionOwner` namespace could be modified by the `collection` Admin only;
		/// - `ItemOwner` namespace could be modified by the `maybe_item` owner only. `maybe_item`
		///   should be set in that case;
		/// - `Account(AccountId)` namespace could be modified only when the `origin` was given a
		///   permission to do so;
		///
		/// The funds of `origin` are reserved according to the formula:
		/// `AttributeDepositBase + DepositPerByte * (key.len + value.len)` taking into
		/// account any already reserved funds.
		///
		/// - `collection`: The identifier of the collection whose item's metadata to set.
		/// - `maybe_item`: The identifier of the item whose metadata to set.
		/// - `namespace`: Attribute's namespace.
		/// - `key`: The key of the attribute.
		/// - `value`: The value to which to set the attribute.
		///
		/// Emits `AttributeSet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(31)]
		#[pallet::weight(T::NftsWeightInfo::set_attribute())]
		pub fn set_attribute(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			maybe_item: Option<T::ItemId>,
			namespace: AttributeNamespace<T::AccountId>,
			key: BoundedVec<u8, T::KeyLimit>,
			value: BoundedVec<u8, T::ValueLimit>,
		) -> DispatchResult {
			pallet_nfts::pallet::Pallet::<T>::set_attribute(
				origin, collection, maybe_item, namespace, key, value,
			)
		}

		/// Clear an attribute for a collection or item.
		///
		/// Simply re-call `clear_attribute` of `pallet-nfts`.
		///
		/// Origin must be either `ForceOrigin` or Signed and the sender should be the Owner of the
		/// attribute.
		///
		/// Any deposit is freed for the collection's owner.
		///
		/// - `collection`: The identifier of the collection whose item's metadata to clear.
		/// - `maybe_item`: The identifier of the item whose metadata to clear.
		/// - `namespace`: Attribute's namespace.
		/// - `key`: The key of the attribute.
		///
		/// Emits `AttributeCleared`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(32)]
		#[pallet::weight(T::NftsWeightInfo::clear_attribute())]
		pub fn clear_attribute(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			maybe_item: Option<T::ItemId>,
			namespace: AttributeNamespace<T::AccountId>,
			key: BoundedVec<u8, T::KeyLimit>,
		) -> DispatchResult {
			pallet_nfts::pallet::Pallet::<T>::clear_attribute(
				origin, collection, maybe_item, namespace, key,
			)
		}

		/// Set the metadata for an item.
		///
		/// Simply re-call `set_metadata` of `pallet-nfts`.
		///
		///
		/// Origin must be either `ForceOrigin` or Signed and the sender should be the Admin of the
		/// `collection`.
		///
		/// If the origin is Signed, then funds of signer are reserved according to the formula:
		/// `MetadataDepositBase + DepositPerByte * data.len` taking into
		/// account any already reserved funds.
		///
		/// - `collection`: The identifier of the collection whose item's metadata to set.
		/// - `item`: The identifier of the item whose metadata to set.
		/// - `data`: The general information of this item. Limited in length by `StringLimit`.
		///
		/// Emits `ItemMetadataSet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(33)]
		#[pallet::weight(T::NftsWeightInfo::set_metadata())]
		pub fn set_metadata(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
			data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResult {
			pallet_nfts::pallet::Pallet::<T>::set_metadata(origin, collection, item, data)
		}

		/// Clear the metadata for an item.
		///
		/// Simply re-call `clear_metadata` of `pallet-nfts`.
		///
		/// Origin must be either `ForceOrigin` or Signed and the sender should be the Admin of the
		/// `collection`.
		///
		/// Any deposit is freed for the collection's owner.
		///
		/// - `collection`: The identifier of the collection whose item's metadata to clear.
		/// - `item`: The identifier of the item whose metadata to clear.
		///
		/// Emits `ItemMetadataCleared`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(34)]
		#[pallet::weight(T::NftsWeightInfo::clear_metadata())]
		pub fn clear_metadata(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
		) -> DispatchResult {
			pallet_nfts::pallet::Pallet::<T>::clear_metadata(origin, collection, item)
		}

		/// Set the metadata for a collection.
		///
		/// Simply re-call `set_collection_metadata` of `pallet-nfts`.
		///
		/// Origin must be either `ForceOrigin` or `Signed` and the sender should be the Admin of
		/// the `collection`.
		///
		/// If the origin is `Signed`, then funds of signer are reserved according to the formula:
		/// `MetadataDepositBase + DepositPerByte * data.len` taking into
		/// account any already reserved funds.
		///
		/// - `collection`: The identifier of the item whose metadata to update.
		/// - `data`: The general information of this item. Limited in length by `StringLimit`.
		///
		/// Emits `CollectionMetadataSet`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(35)]
		#[pallet::weight(T::NftsWeightInfo::set_collection_metadata())]
		pub fn set_collection_metadata(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResult {
			pallet_nfts::pallet::Pallet::<T>::set_collection_metadata(origin, collection, data)
		}

		/// Clear the metadata for a collection.
		///
		/// Simply re-call `clear_collection_metadata` of `pallet-nfts`.
		///
		/// Origin must be either `ForceOrigin` or `Signed` and the sender should be the Admin of
		/// the `collection`.
		///
		/// Any deposit is freed for the collection's owner.
		///
		/// - `collection`: The identifier of the collection whose metadata to clear.
		///
		/// Emits `CollectionMetadataCleared`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(36)]
		#[pallet::weight(T::NftsWeightInfo::clear_collection_metadata())]
		pub fn clear_collection_metadata(
			origin: OriginFor<T>,
			collection: T::CollectionId,
		) -> DispatchResult {
			pallet_nfts::pallet::Pallet::<T>::clear_collection_metadata(origin, collection)
		}

		/// Change the Issuer, Admin and Freezer of a collection.
		///
		/// Simply re-call `set_team` of `pallet-nfts`.
		///
		/// Origin must be either `ForceOrigin` or Signed and the sender should be the Owner of the
		/// `collection`.
		///
		/// Note: by setting the role to `None` only the `ForceOrigin` will be able to change it
		/// after to `Some(account)`.
		///
		/// - `collection`: The collection whose team should be changed.
		/// - `issuer`: The new Issuer of this collection.
		/// - `admin`: The new Admin of this collection.
		/// - `freezer`: The new Freezer of this collection.
		///
		/// Emits `TeamChanged`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(37)]
		#[pallet::weight(T::NftsWeightInfo::set_team())]
		pub fn set_team(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			issuer: Option<AccountIdLookupOf<T>>,
			admin: Option<AccountIdLookupOf<T>>,
			freezer: Option<AccountIdLookupOf<T>>,
		) -> DispatchResult {
			pallet_nfts::pallet::Pallet::<T>::set_team(origin, collection, issuer, admin, freezer)
		}

		// SBP-M2: DispatchResultWithPostInfo should be used for actual `proof_size`.
		// Please refer set_upgrade_item's comment

		/// Create a dynamic mining pool.
		///
		/// Origin must be Signed and the sender should have sufficient items in the `loot_table`.
		///
		/// Note: The mining chance will be changed after each NFT is minted.
		///
		/// - `loot_table`: A bundle of NFTs for mining.
		/// - `admin`: The Admin of this mining pool.
		/// - `mint_settings`: The mining pool settings.
		///
		/// Emits `MiningPoolCreated`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(38)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::create_dynamic_pool(1_u32))]
		pub fn create_dynamic_pool(
			origin: OriginFor<T>,
			loot_table: LootTable<T::CollectionId, T::ItemId>,
			admin: AccountIdLookupOf<T>,
			mint_settings: MintSettingsFor<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let id = Self::get_pool_id();
			let admin = T::Lookup::lookup(admin)?;
			Self::do_create_dynamic_pool(&id, &sender, loot_table, &admin, mint_settings)?;
			Ok(())
		}

		// SBP-M2: DispatchResultWithPostInfo should be used for actual `proof_size`.
		// Please see set_upgrade_item's comment

		/// Create a stable mining pool.
		///
		/// Origin must be Signed and the sender should be the owner of all collections in the
		/// `loot_table`. Collection in `loot_table` must be infinite supply.
		///
		/// Note: The mining chance will not be changed after each NFT is minted.
		///
		/// - `loot_table`: A bundle of NFTs for mining.
		/// - `admin`: The Admin of this mining pool.
		/// - `mint_settings`: The mining pool settings.
		///
		/// Emits `MiningPoolCreated`.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(39)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::create_stable_pool(1_u32))]
		pub fn create_stable_pool(
			origin: OriginFor<T>,
			loot_table: LootTable<T::CollectionId, T::ItemId>,
			admin: AccountIdLookupOf<T>,
			mint_settings: MintSettingsFor<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let id = Self::get_pool_id();
			let admin = T::Lookup::lookup(admin)?;
			Self::do_create_stable_pool(&id, &sender, loot_table, &admin, mint_settings)?;
			Ok(())
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config<I>, I: 'static> ValidateUnsigned for Pallet<T, I> {
		type Call = Call<T, I>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			match call {
				Call::submit_random_seed_unsigned { seed: _ } => match source {
					TransactionSource::Local | TransactionSource::InBlock => {
						let valid_tx = |provide| {
							ValidTransaction::with_tag_prefix("pallet-game")
								.priority(UNSIGNED_TXS_PRIORITY) // please define `UNSIGNED_TXS_PRIORITY` before this line
								.and_provides([&provide])
								.longevity(3)
								.propagate(true)
								.build()
						};
						valid_tx(b"approve_whitelist_unsigned".to_vec())
					},
					_ => InvalidTransaction::Call.into(),
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	fn submit_random_seed_raw_unsigned(_block_number: T::BlockNumber) -> Result<(), &'static str> {
		let random_seed = sp_io::offchain::random_seed();

		let call = Call::submit_random_seed_unsigned { seed: random_seed };

		let _ = SubmitTransaction::<T, Call<T, I>>::submit_unsigned_transaction(call.into())
			.map_err(|_| {
				log::error!("Failed in offchain_unsigned_tx");
			});
		Ok(())
	}

	/// Return `Ok(())` if `who` is the owner of `game`.
	pub fn ensure_game_owner(who: &T::AccountId, game: &T::GameId) -> Result<(), Error<T, I>> {
		match Game::<T, I>::get(game) {
			Some(config) => {
				ensure!(config.owner == *who, Error::<T, I>::NoPermission);
				Ok(())
			},
			None => Err(Error::<T, I>::UnknownGame.into()),
		}
	}

	/// Return `Ok(())` if `who` is the owner of `collection`.
	pub fn ensure_collection_owner(
		who: &T::AccountId,
		collection: &T::CollectionId,
	) -> Result<(), Error<T, I>> {
		if let Some(owner) = T::Nfts::collection_owner(collection) {
			ensure!(owner == who.clone(), Error::<T, I>::NoPermission);
			return Ok(())
		}
		return Err(Error::<T, I>::UnknownCollection)
	}
}
