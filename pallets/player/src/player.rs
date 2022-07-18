use frame_support::RuntimeDebug;
use frame_support::pallet_prelude::*;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use gafi_primitives::constant::ID;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};

use crate::pallet::NAME;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Player<AccountId> {
	pub id: ID,
	pub owner: AccountId,
	pub name: NAME,
}
