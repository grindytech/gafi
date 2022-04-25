use frame_support::RuntimeDebug;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use crate::pallet::{ID, NAME};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Player<AccountId> {
	pub id: ID,
	pub owner: AccountId,
	pub name: NAME,
}
