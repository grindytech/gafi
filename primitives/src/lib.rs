#![cfg_attr(not(feature = "std"), no_std)]

pub mod currency;
use currency::{Token, TokenInfo, NativeToken};

pub struct AuroraNetworkCurrency {}

impl TokenInfo for AuroraNetworkCurrency {
	fn token_info(token: NativeToken) -> Token {
		let aur: Token = Token {
			id: 1,
			name: b"Aurora".to_vec(),
			symbol: "AUR".as_bytes().to_vec(),
			decimals: 12,
		};
		let aux: Token = Token {
			id: 2,
			name: b"Aurora X".to_vec(),
			symbol: "AUX".as_bytes().to_vec(),
			decimals: 12,
		};

		match token {
			NativeToken::AUX => aux,
			NativeToken::AUR => aur,
		}
	}
}

pub fn unit(token: NativeToken) -> u128 {
	10u128.saturating_pow( AuroraNetworkCurrency::token_info(token).decimals.into() )
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
