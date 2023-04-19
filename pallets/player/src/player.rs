use codec::{Decode, Encode};
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use frame_support::{pallet_prelude::*, RuntimeDebug};
use gafi_primitives::common::constant::ID;
use scale_info::TypeInfo;

use crate::pallet::NAME;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Player<AccountId> {
	pub id: ID,
	pub owner: AccountId,
	pub name: NAME,
}
