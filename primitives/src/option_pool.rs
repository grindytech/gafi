
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use frame_support::{
    pallet_prelude::*,
};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub trait PackServiceProvider<Balance> {
	fn get_service(service: PackService) -> Option<Service<Balance>>;
}

pub trait OptionPoolPlayer<AccountId> {
	fn get_option_pool_player(player: &AccountId) -> Option<OptionPlayer<AccountId>>;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
    Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Service<Balance> {
    pub tx_limit: u8, // max number of transaction per minute
    pub discount: u8,
    pub service: Balance,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum PackService {
    Basic,
    Medium,
    Max,
}

// Struct, Enum
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
    Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct OptionPlayer<AccountId> {
    pub address: AccountId,
    pub join_time: u128,
    pub service: PackService,
}
