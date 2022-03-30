#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use frame_support::{
    pallet_prelude::*,
};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;


#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Default, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub struct Token {
    pub name:  Vec<u8>,
    pub symbol:  Vec<u8>,
    pub decimals: u8,
    pub id: u8 
}

pub trait TokenInfo {
    fn token_info(token: NativeToken) -> Token;
}

pub enum NativeToken {
    AUR,
    AUX,
}



