use codec::{Encode, Decode, MaxEncodedLen};
use frame_support::{RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;
use sp_core::Get;

/// Information about a game.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(NameLimit))]
pub struct GameDetails<AccountId, DepositBalance, NameLimit: Get<u32>> {
	/// game's owner.
	pub(super) owner: AccountId,
	/// The total balance deposited by the owner for all the storage data associated with this
	/// game. Used by `destroy`.
	pub(super) owner_deposit: DepositBalance,
	/// The total number of outstanding collections of this game.
	pub(super) collections: u32,
	/// The total number of outstanding collection metadata of this game.
	pub(super) collection_metadatas: u32,
    /// game name
    pub(super) name: BoundedVec<u8, NameLimit>,
}
