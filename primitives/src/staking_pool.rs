#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use frame_support::{
    pallet_prelude::*,
};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub trait StakingPool<AccountId> {
	fn is_staking_pool(player: &AccountId) -> Option<Player<AccountId>>;

	fn staking_pool_discount() -> u8;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Player<AccountId> {
	pub address: AccountId,
	pub join_time: u128,
}


