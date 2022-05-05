#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use frame_support::{
    pallet_prelude::*,
};
use scale_info::TypeInfo;
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
    GAFI,
    GAKI,
}

pub struct GafiCurrency {}

impl TokenInfo for GafiCurrency {
	fn token_info(token: NativeToken) -> Token {
		let gaki: Token = Token {
			id: 1,
			name: b"Gafi Native Token".to_vec(),
			symbol: "GAFI".as_bytes().to_vec(),
			decimals: 18,
		};
		let gafi: Token = Token {
			id: 2,
			name: b"Gaki Native Token".to_vec(),
			symbol: "GAKI".as_bytes().to_vec(),
			decimals: 18,
		};

		match token {
			NativeToken::GAKI => gaki,
			NativeToken::GAFI => gafi,
		}
	}
}

pub fn unit(token: NativeToken) -> u128 {
	10u128.saturating_pow( GafiCurrency::token_info(token).decimals.into() )
}

/// 1 centi = 0.01 unit
pub fn centi(token: NativeToken) -> u128 {
    unit(token) / 100
}

/// 1 milli = 0.001 unit
pub fn milli(token: NativeToken) -> u128 {
    unit(token) / 1000
}

/// 1 millicent = 0.00001 unit
pub fn millicent(token: NativeToken) -> u128 {
    centi(token) / 1000
}

/// 1 microcent = 0.000001 unit
pub fn microcent(token: NativeToken) -> u128 {
    milli(token) / 1000
}
