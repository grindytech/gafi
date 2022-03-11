
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use frame_support::{
    pallet_prelude::*,
};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::pallet::{Config, BalanceOf};

pub trait PackServiceProvider<T: Config> {
	fn get_service(service: PackService) -> Option<Service<T>>;
}
pub trait AuroraZone<T: Config> {
	fn is_in_aurora_zone(player: &T::AccountId) -> Option<Player<T>>;
}



#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Service<T: Config> {
    pub tx_limit: u8, // max number of transaction per minute
    pub discount: u8,
    pub service: BalanceOf<T>,
}

impl<T: Config> MaxEncodedLen for Service<T> {
    fn max_encoded_len() -> usize {
        1000
    }
}

#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum PackService {
    Basic,
    Medium,
    Max,
}

impl MaxEncodedLen for PackService {
    fn max_encoded_len() -> usize {
        1000
    }
}


// Struct, Enum
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Player<T: Config> {
    pub address: T::AccountId,
    pub join_block: u64,
    pub service: PackService,
}

impl<T: Config> MaxEncodedLen for Player<T> {
    fn max_encoded_len() -> usize {
        1000
    }
}

