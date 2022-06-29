use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_runtime::{RuntimeDebug, Permill};

use crate::constant::ID;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Service {
	pub tx_limit: u32, // max number of discounted transaction user can use in TimeService
	pub discount: Permill,  // percentage of discount
}

pub trait MasterPool<AccountId> {
	fn remove_player(player: &AccountId, pool_id: ID);
	fn get_timeservice() -> u128;
	fn get_marktime() -> u128;
}

impl<AccountId> MasterPool<AccountId> for () {
	fn remove_player(_player: &AccountId, _pool_id: ID) {}
	fn get_timeservice() -> u128 {
		30 * 60_000u128 // 30 minutes
	}
	fn get_marktime() -> u128 {
		u128::default()
	}
}
