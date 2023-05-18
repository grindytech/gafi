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

use codec::{MaxEncodedLen};
use frame_support::{
	ensure,
	traits::{
		tokens::nonfungibles_v2::{Create, Inspect, Mutate, Transfer},
		Currency, Randomness, ReservableCurrency,
	},
	PalletId,
};
use frame_system::{
	offchain::{CreateSignedTransaction, SubmitTransaction},
	Config as SystemConfig,
};
use gafi_support::game::{
	CreateCollection, CreateItem, GameSetting, MutateItem, TransferItem, UpgradeItem,
};
use pallet_nfts::{CollectionConfig, Incrementable, ItemConfig};
use sp_core::offchain::KeyTypeId;
use sp_runtime::{
	traits::{StaticLookup, TrailingZeroInput},
	Percent,
};
use sp_std::vec::Vec;
use types::{GameDetails, ItemUpgradeConfig};

pub type BalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

pub type BlockNumber<T> = <T as SystemConfig>::BlockNumber;

type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;

pub type GameDetailsFor<T, I> = GameDetails<<T as SystemConfig>::AccountId, BalanceOf<T, I>>;

pub type CollectionConfigFor<T, I = ()> =
	CollectionConfig<BalanceOf<T, I>, BlockNumber<T>, <T as pallet_nfts::Config>::CollectionId>;

pub type ItemUpgradeConfigFor<T, I = ()> =
	ItemUpgradeConfig<<T as pallet_nfts::Config>::ItemId, BalanceOf<T, I>>;

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
		pallet_prelude::{ValueQuery, *},
		Blake2_128Concat, Twox64Concat,
	};
	use frame_system::pallet_prelude::{OriginFor, *};
	use gafi_support::game::Level;
	use pallet_nfts::CollectionRoles;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(_);

	pub type BalanceOf<T, I = ()> =
		<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

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
	pub(super) type Game<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::GameId, GameDetailsFor<T, I>>;

	#[pallet::storage]
	pub(super) type NextGameId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::GameId, OptionQuery>;

	#[pallet::storage]
	pub(super) type SwapFee<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::GameId, (Percent, BlockNumber<T>)>;

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
	pub(super) type ItemBalances<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, T::CollectionId>,
			NMapKey<Twox64Concat, T::ItemId>,
		),
		u32,
		ValueQuery,
	>;

	/// Item reserve created by the owner, random mining by player
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
	pub(super) type TotalReserveOf<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		T::CollectionId,
		u32,
		ValueQuery,
	>;

	/// Game collection config
	#[pallet::storage]
	pub(super) type GameCollectionConfigOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::CollectionId, CollectionConfigFor<T, I>, OptionQuery>;

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

	/// Store random seed generated from the off-chain worker per block
	#[pallet::storage]
	pub(crate) type RandomSeed<T: Config<I>, I: 'static = ()> =
		StorageValue<_, [u8; 32], ValueQuery>;

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
		UnknownUpgrade,
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

			Self::do_set_upgrade_item(&sender, &collection, &item, &new_item, &config, level, fee)?;

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

		#[pallet::call_index(13)]
		#[pallet::weight(0)]
		pub fn submit_random_seed_unsigned(origin: OriginFor<T>, seed: [u8; 32]) -> DispatchResult {
			ensure_none(origin)?;

			RandomSeed::<T, I>::set(seed);

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
}
