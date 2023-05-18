use codec::{Decode, Encode, MaxEncodedLen};
use core::primitive::u32;
use frame_support::{RuntimeDebug};
use scale_info::TypeInfo;

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

	pub fn minus(mut self, amount: u32) -> Self {
		self.amount -= amount;
		self
	}
}

/// Item upgrade configuration.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ItemUpgradeConfig<ItemId, Price> {

	pub item: ItemId,

	pub fee: Price,
}
