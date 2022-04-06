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

pub struct GafiCurrency {}

impl TokenInfo for GafiCurrency {
	fn token_info(token: NativeToken) -> Token {
		let aur: Token = Token {
			id: 1,
			name: b"Aurora".to_vec(),
			symbol: "AUR".as_bytes().to_vec(),
			decimals: 18,
		};
		let aux: Token = Token {
			id: 2,
			name: b"Aurora X".to_vec(),
			symbol: "AUX".as_bytes().to_vec(),
			decimals: 18,
		};

		match token {
			NativeToken::AUX => aux,
			NativeToken::AUR => aur,
		}
	}
}

pub fn unit(token: NativeToken) -> u128 {
	10u128.saturating_pow( GafiCurrency::token_info(token).decimals.into() )
}

pub fn centi(token: NativeToken) -> u128 {
    unit(token) / 100
}

pub fn milli(token: NativeToken) -> u128 {
    unit(token) / 1000
}

pub fn millicent(token: NativeToken) -> u128 {
    centi(token) / 1000
}

pub fn microcent(token: NativeToken) -> u128 {
    milli(token) / 1000
}
