use crate::*;
use codec::{Decode, Encode};
use core::primitive::u32;
use frame_support::{
	pallet_prelude::{MaxEncodedLen},
	RuntimeDebug,
};

use scale_info::TypeInfo;
pub type BalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

pub type BlockNumber<T> = <T as SystemConfig>::BlockNumber;

pub type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;

pub type GameDetailsFor<T, I> = GameDetails<<T as SystemConfig>::AccountId, BalanceOf<T, I>>;

pub type CollectionConfigFor<T, I = ()> =
	CollectionConfig<BalanceOf<T, I>, BlockNumber<T>, <T as pallet_nfts::Config>::CollectionId>;

pub type ItemUpgradeConfigFor<T, I = ()> =
	UpgradeItemConfig<<T as pallet_nfts::Config>::ItemId, BalanceOf<T, I>>;

pub(crate) type PackageFor<T> =
	Package<<T as pallet_nfts::Config>::CollectionId, <T as pallet_nfts::Config>::ItemId>;

/// Information about a game.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(NameLimit))]
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

#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(NameLimit))]
pub struct Item<ItemId> {
	pub item: ItemId,
	pub amount: u32,
}

impl<ItemId> Item<ItemId> {
	pub fn new(item: ItemId, amount: u32) -> Self {
		Item { item, amount }
	}

	pub fn sub(mut self, amount: u32) -> Self {
		self.amount -= amount;
		self
	}
}

/// Upgrade Item configuration.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct UpgradeItemConfig<ItemId, Price> {
	pub item: ItemId,

	pub fee: Price,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct BundleConfig<AccountId, Price> {
	pub owner: AccountId,
	pub price: Price,
}
