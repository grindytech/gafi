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
    //  {
    //     let aur: Token = Token { id: 1, name: b"Aurora".to_vec(), symbol: "AUR".as_bytes().to_vec(),  decimals: 12, };
    //     let aux: Token = Token { id: 2, name: b"Aurora X".to_vec(), symbol: "AUX".as_bytes().to_vec(),  decimals: 12, };
        
    //     match currency {
    //         Currency::AUR => Some(aur),
    //         Currency::AUX => Some(aux),
    //     }
    // }
}

pub enum NativeToken {
    AUR,
    AUX,
}



