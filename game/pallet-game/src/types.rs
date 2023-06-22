use crate::*;
use codec::{Decode, Encode};
use core::primitive::u32;
use frame_support::{
	pallet_prelude::{BoundedVec, MaxEncodedLen},
	RuntimeDebug,
};
use gafi_support::game::{Loot, MintSettings};

use scale_info::TypeInfo;
pub type BalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

pub type BlockNumber<T> = <T as SystemConfig>::BlockNumber;

pub type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;

pub type GameDetailsFor<T, I> = GameDetails<<T as SystemConfig>::AccountId, BalanceOf<T, I>>;

pub type TradeConfigFor<T, I> =
	TradeConfig<<T as SystemConfig>::AccountId, BalanceOf<T, I>, BundleFor<T, I>, BlockNumber<T>>;

pub type PoolDetailsFor<T, I> = PoolDetails<
	<T as SystemConfig>::AccountId,
	BalanceOf<T, I>,
	BlockNumber<T>,
	<T as pallet_nfts::Config>::CollectionId,
>;

pub type CollectionConfigFor<T, I = ()> =
	CollectionConfig<BalanceOf<T, I>, BlockNumber<T>, <T as pallet_nfts::Config>::CollectionId>;

pub type ItemUpgradeConfigFor<T, I = ()> =
	UpgradeItemConfig<<T as pallet_nfts::Config>::ItemId, BalanceOf<T, I>>;

pub type MintSettingsFor<T, I = ()> =
	MintSettings<BalanceOf<T, I>, BlockNumber<T>, <T as pallet_nfts::Config>::CollectionId>;

pub(crate) type BundleFor<T, I = ()> = BoundedVec<
	Package<<T as pallet_nfts::Config>::CollectionId, <T as pallet_nfts::Config>::ItemId>,
	<T as pallet::Config<I>>::MaxBundle,
>;

pub(crate) type LootTableFor<T, I = ()> = BoundedVec<
	Loot<<T as pallet_nfts::Config>::CollectionId, <T as pallet_nfts::Config>::ItemId>,
	<T as pallet::Config<I>>::MaxLoot,
>;

/// Information about a game.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct GameDetails<AccountId, DepositBalance> {
	/// game's owner.
	pub(super) owner: AccountId,
	/// The total balance deposited by the owner for all the storage data associated with this
	/// game. Used by `destroy`.
	pub(super) owner_deposit: DepositBalance,
	/// The total number of outstanding collections of this game.
	pub(super) collections: u32,
	/// Can thaw tokens, force transfers and burn tokens from any account.
	pub(super) admin: AccountId,
}

/// Upgrade Item configuration.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct UpgradeItemConfig<ItemId, Price> {
	pub item: ItemId,
	pub fee: Price,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct TradeConfig<AccountId, Price, Bundle, BlockNumber> {
	pub trade: TradeType,
	pub owner: AccountId,
	pub maybe_price: Option<Price>,
	pub maybe_required: Option<Bundle>,
	pub start_block: Option<BlockNumber>,
	pub end_block: Option<BlockNumber>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AuctionConfig<AccountId, Price, BlockNumber> {
	pub owner: AccountId,
	pub maybe_price: Option<Price>,
	pub start_block: BlockNumber,
	pub duration: BlockNumber,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ItemBalanceStatus {
	Reserved,
	Free,
}

/// Types of the mining pool
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum PoolType {
	/// Item mining chance will change depending on item supply.
	Dynamic,
	/// Item mining chance is fixed with an infinite supply.
	Stable,
}

/// Information about a mining pool.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct PoolDetails<AccountId, Balance, BlockNumber, CollectionId> {
	/// pool type
	pub(super) pool_type: PoolType,
	/// game's owner.
	pub(super) owner: AccountId,
	/// The total balance deposited by the owner for all the storage data associated with this
	/// game. Used by `destroy`.
	pub(super) owner_deposit: Balance,
	/// Can create a new pool, add more resources.
	pub(super) admin: AccountId,
	/// mint settings
	pub(super) mint_settings: MintSettings<Balance, BlockNumber, CollectionId>,
}
