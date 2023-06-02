#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod features;
mod types;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;
use crate::weights::WeightInfo;
pub use weights::*;

use codec::MaxEncodedLen;
use frame_support::{
	ensure,
	traits::{
		tokens::nonfungibles_v2::{Create, Inspect, Mutate, Transfer},
		Currency, Randomness, ReservableCurrency,
	},
	transactional, PalletId,
};
use frame_system::{
	offchain::{CreateSignedTransaction, SubmitTransaction},
	Config as SystemConfig,
};
use gafi_support::game::{
	Auction, CreateItem, GameSetting, Level, MutateCollection, MutateItem, Package, Swap, Trade,
	TransferItem, UpgradeItem, Wishlist,
};
use pallet_nfts::{CollectionConfig, Incrementable, ItemConfig};
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
	use crate::types::Item;

	use super::*;
	use frame_support::{
		pallet_prelude::{OptionQuery, ValueQuery, *},
		Blake2_128Concat, Twox64Concat,
	};
	use frame_system::pallet_prelude::{OriginFor, *};
	use gafi_support::game::Bundle;
	use pallet_nfts::CollectionRoles;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(_);

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<GameId, TradeId, BlockNumber> {
		fn game(i: u16) -> GameId;

		fn trade(i: u16) -> TradeId;

		fn block(i: u16) -> BlockNumber;
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<GameId: From<u16>, TradeId: From<u16>, BlockNumber: From<u16>>
		BenchmarkHelper<GameId, TradeId, BlockNumber> for ()
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

		/// The currency mechanism, used for paying for reserves.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// pallet_nfts
		type Nfts: Mutate<Self::AccountId, ItemConfig>
			+ Transfer<Self::AccountId>
			+ Create<
				Self::AccountId,
				CollectionConfig<BalanceOf<Self, I>, Self::BlockNumber, Self::CollectionId>,
			> + Inspect<Self::AccountId>
			+ Inspect<Self::AccountId, ItemId = Self::ItemId, CollectionId = Self::CollectionId>;

		/// generate random ID
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// The type used to identify a unique game
		type GameId: Member + Parameter + MaxEncodedLen + Copy + Incrementable;

		/// The type used to identify a unique trade
		type TradeId: Member + Parameter + MaxEncodedLen + Copy + Incrementable;

		/// The basic amount of funds that must be reserved for game.
		#[pallet::constant]
		type GameDeposit: Get<BalanceOf<Self, I>>;

		/// Max number of collections in a game
		#[pallet::constant]
		type MaxGameCollection: Get<u32>;

		/// Maximum number of item that a collection could has
		#[pallet::constant]
		type MaxItem: Get<u32>;

		/// Maximum number of item minted once
		#[pallet::constant]
		type MaxMintItem: Get<u32>;

		/// The basic amount of funds that must be reserved for any upgrade.
		#[pallet::constant]
		type UpgradeDeposit: Get<BalanceOf<Self, I>>;

		/// The basic amount of funds that must be reserved for any sale.
		#[pallet::constant]
		type SaleDeposit: Get<BalanceOf<Self, I>>;

		/// Maximum collection in a bundle
		#[pallet::constant]
		type MaxBundle: Get<u32>;

		/// The basic amount of funds that must be reserved for any bundle.
		#[pallet::constant]
		type BundleDeposit: Get<BalanceOf<Self, I>>;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type Helper: BenchmarkHelper<Self::GameId, Self::TradeId, Self::BlockNumber>;
	}

	/// Store basic game info
	#[pallet::storage]
	pub(super) type Game<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::GameId, GameDetailsFor<T, I>>;

	#[pallet::storage]
	pub(super) type NextGameId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::GameId, OptionQuery>;

	/// Collections in the game
	#[pallet::storage]
	pub(super) type CollectionsOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		T::GameId,
		BoundedVec<T::CollectionId, T::MaxGameCollection>,
		ValueQuery,
	>;

	/// Storing Collection Minting Fee
	#[pallet::storage]
	pub(super) type MintingFeeOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::CollectionId, BalanceOf<T, I>, OptionQuery>;

	/// Collection belongs to
	#[pallet::storage]
	pub(super) type GameOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::CollectionId, T::GameId, OptionQuery>;

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

	/// Storing lock balance
	#[pallet::storage]
	pub(super) type LockBalanceOf<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, T::CollectionId>,
			NMapKey<Twox64Concat, T::ItemId>,
		),
		u32,
		ValueQuery,
	>;

	/// Item reserve for random minting created by the owner
	#[pallet::storage]
	pub(super) type ItemReserve<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		T::CollectionId,
		BoundedVec<Item<T::ItemId>, T::MaxItem>,
		ValueQuery,
	>;

	/// Item reserve created by the owner, random mining by player
	#[pallet::storage]
	pub(super) type TotalReserveOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::CollectionId, u32, ValueQuery>;

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

	/// Store the original items of the upgraded items
	#[pallet::storage]
	pub(super) type OriginItemOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		(T::CollectionId, T::ItemId),
		(T::CollectionId, T::ItemId),
		OptionQuery,
	>;

	/// Store the upgrade config
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

	/// Storing next bundle id
	#[pallet::storage]
	pub(super) type NextTradeId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::TradeId, OptionQuery>;

	/// Storing package
	#[pallet::storage]
	pub(super) type PackageOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::TradeId,
		Package<T::CollectionId, T::ItemId>,
		OptionQuery,
	>;

	/// Storing bundle
	#[pallet::storage]
	pub(super) type BundleOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::TradeId, BundleFor<T, I>, ValueQuery>;

	/// Storing trade configuration
	#[pallet::storage]
	pub(super) type TradeConfigOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::TradeId,
		TradeConfig<T::AccountId, BalanceOf<T, I>, BundleFor<T, I>>,
		OptionQuery,
	>;

	/// Storing auction configuration
	#[pallet::storage]
	pub(super) type AuctionConfigOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::TradeId,
		AuctionConfig<T::AccountId, BalanceOf<T, I>, T::BlockNumber>,
		OptionQuery,
	>;

	#[pallet::storage]
	pub(super) type BidWinnerOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::TradeId, (T::AccountId, BalanceOf<T, I>), OptionQuery>;

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
		ItemCreated {
			who: T::AccountId,
			collection: T::CollectionId,
			item: T::ItemId,
			amount: u32,
		},
		ItemAdded {
			who: T::AccountId,
			collection: T::CollectionId,
			item: T::ItemId,
			amount: u32,
		},
		Minted {
			who: T::AccountId,
			target: T::AccountId,
			collection: T::CollectionId,
			items: Vec<T::ItemId>,
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
			price: BalanceOf<T, I>,
		},
		ItemBought {
			trade: T::TradeId,
			seller: T::AccountId,
			buyer: T::AccountId,
			collection: T::CollectionId,
			item: T::ItemId,
			amount: u32,
			price: BalanceOf<T, I>,
		},
		BundleSet {
			trade: T::TradeId,
			who: T::AccountId,
			bundle: Bundle<T::CollectionId, T::ItemId>,
			price: BalanceOf<T, I>,
		},
		BundleBought {
			trade: T::TradeId,
			seller: T::AccountId,
			buyer: T::AccountId,
			price: BalanceOf<T, I>,
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
			wisher: T::AccountId,
			filler: T::AccountId,
			price: BalanceOf<T, I>,
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
		},
		AuctionSet {
			trade: T::TradeId,
			who: T::AccountId,
			source: Bundle<T::CollectionId, T::ItemId>,
			maybe_price: Option<BalanceOf<T, I>>,
			start_block: T::BlockNumber,
			duration: T::BlockNumber,
		},
		Bade {
			trade: T::TradeId,
			who: T::AccountId,
			bid: BalanceOf<T, I>,
		},
		AuctionClaimed {
			trade: T::TradeId,
			maybe_bid: Option<(T::AccountId, BalanceOf<T, I>)>,
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

		/// Exceed the maximum allowed item in a collection
		ExceedMaxItem,
		/// The number minted items require exceeds the available items in the reserve
		ExceedTotalAmount,
		/// The number minted items require exceeds the amount allowed per tx
		ExceedAllowedAmount,
		/// Exceed the maximum allowed collection in a game
		ExceedMaxCollection,
		/// Exceed max collections in a bundle
		ExceedMaxBundle,

		SoldOut,
		/// Too many attempts
		WithdrawReserveFailed,
		UpgradeExists,
		/// Add the same collection into a game
		CollectionExists,

		InsufficientItemBalance,
		InsufficientLockBalance,
		/// item amount = 0
		InvalidAmount,

		ItemLocked,
		NotForSale,
		BidTooLow,
		AskTooHigh,
		TradeIdInUse,
		TooLow,

		/// auction
		AuctionInProgress,
		AuctionNotStarted,
		AuctionEnded,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn offchain_worker(_block_number: BlockNumberFor<T>) {
			let _ = Self::submit_random_seed_raw_unsigned(_block_number);
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::create_game(1_u32))]
		#[transactional]
		pub fn create_game(origin: OriginFor<T>, admin: AccountIdLookupOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let admin = T::Lookup::lookup(admin)?;

			let game = NextGameId::<T, I>::get().unwrap_or(T::GameId::initial_value());
			Self::do_create_game(&sender, &game, &admin)?;
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::create_game_collection(1_u32))]
		#[transactional]
		pub fn create_game_collection(
			origin: OriginFor<T>,
			game: T::GameId,
			fee: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_create_game_collection(&sender, &game, fee)?;
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		#[transactional]
		pub fn create_collection(
			origin: OriginFor<T>,
			admin: T::AccountId,
			// config: CollectionConfigFor<T, I>,
			fee: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_create_collection(&sender, &admin, fee)?;

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(0)]
		#[transactional]
		pub fn add_game_collection(
			origin: OriginFor<T>,
			game: T::GameId,
			collection: T::CollectionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_add_collection(&sender, &game, &collection)?;
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::create_item(1_u32))]
		#[transactional]
		pub fn create_item(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
			config: ItemConfig,
			amount: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::do_create_item(&sender, &collection, &item, &config, amount)?;

			Ok(())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::add_item(1_u32))]
		#[transactional]
		pub fn add_item(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
			amount: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::do_add_item(&sender, &collection, &item, amount)?;

			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::mint(1_u32))]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			mint_to: AccountIdLookupOf<T>,
			amount: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let target = T::Lookup::lookup(mint_to)?;

			Self::do_mint(&sender, &collection, &target, amount)?;

			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::burn(1_u32))]
		#[transactional]
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

		#[pallet::call_index(10)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::transfer(1_u32))]
		#[transactional]
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

		#[pallet::call_index(11)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_upgrade_item(1_u32))]
		#[transactional]
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

		#[pallet::call_index(12)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::upgrade_item(1_u32))]
		#[transactional]
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

		#[pallet::call_index(13)]
		#[pallet::weight(0)]
		#[transactional]
		pub fn submit_random_seed_unsigned(origin: OriginFor<T>, seed: [u8; 32]) -> DispatchResult {
			ensure_none(origin)?;

			RandomSeed::<T, I>::set(seed);

			Ok(())
		}

		#[pallet::call_index(14)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_price(1_u32))]
		#[transactional]
		pub fn set_price(
			origin: OriginFor<T>,
			package: Package<T::CollectionId, T::ItemId>,
			price: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let trade = Self::get_trade_id();
			Self::do_set_price(&trade, &sender, package, price)?;

			Ok(())
		}

		#[pallet::call_index(15)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::buy_item(1_u32))]
		#[transactional]
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

		#[pallet::call_index(16)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_bundle(1_u32))]
		#[transactional]
		pub fn set_bundle(
			origin: OriginFor<T>,
			bundle: Bundle<T::CollectionId, T::ItemId>,
			price: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let trade = Self::get_trade_id();

			Self::do_set_bundle(&trade, &sender, bundle, price)?;

			Ok(())
		}

		#[pallet::call_index(17)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::buy_bundle(1_u32))]
		#[transactional]
		pub fn buy_bundle(
			origin: OriginFor<T>,
			trade: T::TradeId,
			bid_price: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::do_buy_bundle(&trade, &sender, bid_price)?;
			Ok(())
		}

		#[pallet::call_index(18)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::cancel_set_price(1_u32))]
		#[transactional]
		pub fn cancel_set_price(origin: OriginFor<T>, trade: T::TradeId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_cancel_price(&trade, &sender)?;
			Ok(())
		}

		#[pallet::call_index(19)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::cancel_set_bundle(1_u32))]
		#[transactional]
		pub fn cancel_set_bundle(origin: OriginFor<T>, trade: T::TradeId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_cancel_bundle(&trade, &sender)?;
			Ok(())
		}

		#[pallet::call_index(20)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_wishlist(1_u32))]
		#[transactional]
		pub fn set_wishlist(
			origin: OriginFor<T>,
			bundle: Bundle<T::CollectionId, T::ItemId>,
			price: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let trade = Self::get_trade_id();
			Self::do_set_wishlist(&trade, &sender, bundle, price)?;
			Ok(())
		}

		#[pallet::call_index(21)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::fill_wishlist(1_u32))]
		#[transactional]
		pub fn fill_wishlist(
			origin: OriginFor<T>,
			trade: T::TradeId,
			ask_price: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_fill_wishlist(&trade, &sender, ask_price)?;
			Ok(())
		}

		#[pallet::call_index(22)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::remove_collection(1_u32))]
		#[transactional]
		pub fn remove_collection(
			origin: OriginFor<T>,
			game: T::GameId,
			collection: T::CollectionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_remove_collection(&sender, &game, &collection)?;
			Ok(())
		}

		#[pallet::call_index(23)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::lock_item_transfer(1_u32))]
		#[transactional]
		pub fn lock_item_transfer(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
		) -> DispatchResult {
			pallet_nfts::pallet::Pallet::<T>::lock_item_transfer(origin, collection, item)
		}

		#[pallet::call_index(24)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::unlock_item_transfer(1_u32))]
		#[transactional]
		pub fn unlock_item_transfer(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
		) -> DispatchResult {
			pallet_nfts::pallet::Pallet::<T>::unlock_item_transfer(origin, collection, item)
		}

		#[pallet::call_index(25)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_swap(1_u32))]
		#[transactional]
		pub fn set_swap(
			origin: OriginFor<T>,
			source: Bundle<T::CollectionId, T::ItemId>,
			required: Bundle<T::CollectionId, T::ItemId>,
			maybe_price: Option<BalanceOf<T, I>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let trade = Self::get_trade_id();
			Self::do_set_swap(&trade, &sender, source, required, maybe_price)?;
			Ok(())
		}

		#[pallet::call_index(26)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::claim_swap(1_u32))]
		#[transactional]
		pub fn claim_swap(
			origin: OriginFor<T>,
			trade: T::TradeId,
			maybe_bid_price: Option<BalanceOf<T, I>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_claim_swap(&trade, &sender, maybe_bid_price)?;
			Ok(())
		}

		#[pallet::call_index(27)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::set_auction(1_u32))]
		#[transactional]
		pub fn set_auction(
			origin: OriginFor<T>,
			source: Bundle<T::CollectionId, T::ItemId>,
			maybe_price: Option<BalanceOf<T, I>>,
			start_block: T::BlockNumber,
			duration: T::BlockNumber,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let trade = Self::get_trade_id();

			Self::do_set_auction(
				&trade,
				&sender,
				source,
				maybe_price,
				start_block,
				duration,
			)?;
			Ok(())
		}

		#[pallet::call_index(28)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::bid_auction(1_u32))]
		#[transactional]
		pub fn bid_auction(
			origin: OriginFor<T>,
			trade: T::TradeId,
			bid: BalanceOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_bid_auction(&trade, &sender, bid)?;
			Ok(())
		}

		#[pallet::call_index(29)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::claim_auction(1_u32))]
		#[transactional]
		pub fn claim_auction(origin: OriginFor<T>, trade: T::TradeId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			Self::do_claim_auction(&trade)?;
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

	pub fn ensure_game_owner(who: &T::AccountId, game: &T::GameId) -> Result<(), Error<T, I>> {
		match Game::<T, I>::get(game) {
			Some(config) => {
				ensure!(config.owner == *who, Error::<T, I>::NoPermission);
				Ok(())
			},
			None => Err(Error::<T, I>::UnknownGame.into()),
		}
	}

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
