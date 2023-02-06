use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_runtime::{Permill, RuntimeDebug};
use sp_std::vec::Vec;

use crate::constant::ID;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum PoolType {
	Upfront,
	Staking,
	Funding,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Service {
	pub tx_limit: u32, // max number of discounted transaction user can use in TimeService
	pub discount: Permill, // percentage of discount
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

// funding pool external service

#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum FundingPoolJoinType {
	Default,
	Whitelist,
}
pub trait FundingPoolJoinTypeHandle<AccountId> {
	fn set_join_type(
		pool_id: ID,
		join_type: FundingPoolJoinType,
		call_check_url: Vec<u8>,
		account_id: AccountId,
	) -> DispatchResult;
	fn reset(pool_id: ID, account_id: AccountId) -> DispatchResult;
	fn get_join_type(pool_id: ID) -> Option<(FundingPoolJoinType, Vec<u8>)>;
}
pub trait GetFundingPoolJoinType {
	fn get_join_type(pool_id: ID) -> (FundingPoolJoinType, Vec<u8>);
}
