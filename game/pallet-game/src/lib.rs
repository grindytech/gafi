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
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::traits::{
	tokens::nonfungibles_v2::{Create, Inspect, Mutate, Transfer},
	Currency, Randomness, ReservableCurrency,
};
use frame_system::Config as SystemConfig;
use gafi_support::game::{CreateCollection, CreateItem, GameSetting};
use pallet_nfts::{CollectionConfig, Incrementable, ItemConfig};
use sp_runtime::{traits::StaticLookup, Percent};
use sp_std::vec::Vec;
use types::GameDetails;

pub type DepositBalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

pub type BalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

pub type BlockNumber<T> = <T as SystemConfig>::BlockNumber;

type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;

// type InspectCollectionId<T, I = ()> = <pallet_nfts::pallet::Pallet<T, I> as Inspect<<T as
// SystemConfig>::AccountId>>::CollectionId;

pub type GameDetailsFor<T, I> = GameDetails<<T as SystemConfig>::AccountId, DepositBalanceOf<T, I>>;

pub type CollectionConfigFor<T, I = ()> =
	CollectionConfig<BalanceOf<T, I>, BlockNumber<T>, <T as pallet_nfts::Config>::CollectionId>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::{OriginFor, *};
	use pallet_nfts::CollectionRoles;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(_);

	pub type BalanceOf<T, I = ()> =
		<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_nfts::Config {
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
				CollectionConfig<DepositBalanceOf<Self, I>, Self::BlockNumber, Self::CollectionId>,
			> + frame_support::traits::tokens::nonfungibles_v2::Inspect<Self::AccountId>
			+ Inspect<Self::AccountId, ItemId = Self::ItemId, CollectionId = Self::CollectionId>;

		/// generate random ID
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// The type used to identify a unique game
		type GameId: Member + Parameter + MaxEncodedLen + Copy + Incrementable;

		/// The basic amount of funds that must be reserved for game.
		#[pallet::constant]
		type GameDeposit: Get<DepositBalanceOf<Self, I>>;

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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		GameCreated { id: T::GameId },
		SwapFeeSetted { id: T::GameId, fee: Percent },
		CollectionCreated { id: T::CollectionId },
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
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn create_game(origin: OriginFor<T>, admin: Option<T::AccountId>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let game_id = NextGameId::<T, I>::get().unwrap_or(T::GameId::initial_value());
			Self::do_create_game(game_id, sender, admin)?;
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
			Self::do_set_swap_fee(game_id, sender, fee, start_block)?;
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn create_game_colletion(
			origin: OriginFor<T>,
			game_id: T::GameId,
			maybe_admin: Option<T::AccountId>,
			config: CollectionConfigFor<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_create_game_collection(sender, game_id, maybe_admin, config)?;
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn create_collection(
			origin: OriginFor<T>,
			maybe_admin: Option<T::AccountId>,
			config: CollectionConfigFor<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::do_create_collection(sender, maybe_admin, config)?;
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
			Self::do_add_collection(sender, game, collection)?;
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

			Self::do_create_item(sender, collection, item, config, amount)?;

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

			Self::do_add_item(sender, collection, item, amount)?;

			Ok(())
		}
	}
}
