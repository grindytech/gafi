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

use frame_support::{
	traits::{
		tokens::nonfungibles_v2::{Create, Inspect, Mutate, Transfer},
		Currency, Randomness, ReservableCurrency,
	},
	PalletId,
};
use frame_system::Config as SystemConfig;
use gafi_support::game::{
	CreateCollection, CreateItem, GameSetting, MutateItem, TransferItem, UpgradeItem,
};
use pallet_nfts::{CollectionConfig, Incrementable, ItemConfig};
use sp_runtime::{traits::StaticLookup, Percent};
use sp_std::vec::Vec;
use types::{GameCollectionConfig, GameDetails, ItemUpgradeConfig};

pub type BalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

pub type BlockNumber<T> = <T as SystemConfig>::BlockNumber;

type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;

// type InspectCollectionId<T, I = ()> = <pallet_nfts::pallet::Pallet<T, I> as Inspect<<T as
// SystemConfig>::AccountId>>::CollectionId;

pub type GameDetailsFor<T, I> = GameDetails<<T as SystemConfig>::AccountId, BalanceOf<T, I>>;

pub type CollectionConfigFor<T, I = ()> =
	GameCollectionConfig<BalanceOf<T, I>, BlockNumber<T>, <T as pallet_nfts::Config>::CollectionId>;

pub type ItemUpgradeConfigFor<T, I = ()> = ItemUpgradeConfig<
	<T as pallet_nfts::Config>::ItemId,
	BalanceOf<T, I>,
>;

#[frame_support::pallet]
pub mod pallet {
	use crate::types::Item;

	use super::*;
	use frame_support::{pallet_prelude::*, Blake2_128Concat, Twox64Concat};
	use frame_system::pallet_prelude::{OriginFor, *};
	use gafi_support::game::{Level};
	use pallet_nfts::CollectionRoles;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(_);

	pub type BalanceOf<T, I = ()> =
		<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_nfts::Config {
		/// The Game's pallet id
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism, used for paying for reserves.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// pallet_nfts
		type Nfts: Mutate<Self::AccountId, ItemConfig>
			+ Transfer<Self::AccountId>
			+ Create<
				Self::AccountId,
				CollectionConfig<BalanceOf<Self, I>, Self::BlockNumber, Self::CollectionId>,
			> + frame_support::traits::tokens::nonfungibles_v2::Inspect<Self::AccountId>
			+ Inspect<Self::AccountId, ItemId = Self::ItemId, CollectionId = Self::CollectionId>;

		/// generate random ID
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// The type used to identify a unique game
		type GameId: Member + Parameter + MaxEncodedLen + Copy + Incrementable;

		/// The basic amount of funds that must be reserved for game.
		#[pallet::constant]
		type GameDeposit: Get<BalanceOf<Self, I>>;

		/// Max name length
		#[pallet::constant]
		type MaxNameLength: Get<u32>;

		/// Min name length
		#[pallet::constant]
		type MinNameLength: Get<u32>;

		/// Max Swapping Fee
		#[pallet::constant]
		type MaxSwapFee: Get<Percent>;

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
	}

	/// Store basic game info
	#[pallet::storage]
	pub(super) type Games<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::GameId, GameDetailsFor<T, I>>;

	#[pallet::storage]
	pub(super) type NextGameId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::GameId, OptionQuery>;

	#[pallet::storage]
	pub(super) type SwapFee<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::GameId, (Percent, BlockNumber<T>)>;

	#[pallet::storage]
	pub(super) type GameCollections<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		T::GameId,
		BoundedVec<T::CollectionId, T::MaxGameCollection>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type CollectionGame<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::CollectionId, T::GameId, OptionQuery>;

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

	#[pallet::storage]
	pub(super) type ItemBalances<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::CollectionId>,
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Twox64Concat, T::ItemId>,
		),
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type ItemReserve<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		T::CollectionId,
		BoundedVec<Item<T::ItemId>, T::MaxItem>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type GameCollectionConfigOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::CollectionId, CollectionConfigFor<T, I>, OptionQuery>;

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

	#[pallet::storage]
	pub(super) type OriginItemOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		(T::CollectionId, T::ItemId),
		(T::CollectionId, T::ItemId),
		OptionQuery,
	>;

	#[pallet::storage]
	pub(super) type UpgradeConfigOf<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::CollectionId>,
			NMapKey<Blake2_128Concat, T::ItemId>,
			NMapKey<Blake2_128Concat, Level>,
		),
		ItemUpgradeConfigFor<T, I>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		GameCreated {
			game_id: T::GameId,
		},
		SwapFeeSetted {
			game_id: T::GameId,
			fee: Percent,
		},
		CollectionCreated {
			collection_id: T::CollectionId,
		},
		ItemCreated {
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			amount: u32,
		},
		ItemAdded {
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			amount: u32,
		},
		Minted {
			minter: T::AccountId,
			target: T::AccountId,
			collection_id: T::CollectionId,
			minted_items: Vec<T::ItemId>,
		},
		Burned {
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			amount: u32,
		},
		Transferred {
			from: T::AccountId,
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			dest: T::AccountId,
			amount: u32,
		},
		Upgraded {
			who: T::AccountId,
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			amount: u32,
		},
		UpgradeSet {
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			level: Level,
		},
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		NotGameOwner,
		UnknownGame,
		NameTooLong,
		NameTooShort,
		SwapFeeTooHigh,
		SwapFeeNotFound,
		NoPermission,
		ExceedMaxCollection,
		UnknownCollection,
		UnknownItem,
		ExceedMaxItem,
		ExceedTotalAmount,
		ExceedAllowedAmount,
		SoldOut,
		WithdrawReserveFailed,
		InsufficientItemBalance,
		NoCollectionConfig,
		UpgradeExists,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn create_game(origin: OriginFor<T>, admin: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let game_id = NextGameId::<T, I>::get().unwrap_or(T::GameId::initial_value());
			Self::do_create_game(&sender, &game_id, &admin)?;
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn set_swap_fee(
			origin: OriginFor<T>,
			game_id: T::GameId,
			fee: Percent,
			start_block: BlockNumber<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_set_swap_fee(&sender, &game_id, fee, start_block)?;
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn create_game_colletion(
			origin: OriginFor<T>,
			game_id: T::GameId,
			admin: T::AccountId,
			config: CollectionConfigFor<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_create_game_collection(&sender, &game_id, &admin, &config)?;
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn create_collection(
			origin: OriginFor<T>,
			admin: T::AccountId,
			config: CollectionConfigFor<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_create_collection(&sender, &admin, &config)?;

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(0)]
		pub fn add_game_collection(
			origin: OriginFor<T>,
			game: T::GameId,
			collection: Vec<T::CollectionId>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_add_collection(&sender, &game, &collection)?;
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(0)]
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
		#[pallet::weight(0)]
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
		#[pallet::weight(0)]
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
		#[pallet::weight(0)]
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
		#[pallet::weight(0)]
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

			Self::deposit_event(Event::<T, I>::Transferred {
				from: sender,
				collection_id: collection,
				item_id: item,
				dest: destination,
				amount,
			});

			Ok(())
		}

		#[pallet::call_index(11)]
		#[pallet::weight(0)]
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

			Self::do_set_upgrade_item(
				&sender,
				&collection,
				&item,
				&new_item,
				&config,
				level,
				fee,
			)?;

			pallet_nfts::pallet::Pallet::<T>::set_metadata(origin, collection, item, data)?;

			Ok(())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(0)]
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
	}
}
