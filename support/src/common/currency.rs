use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_std::prelude::*;

use super::types::Balance;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Default, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub struct Token {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub id: u8,
}

pub trait TokenInfo {
	fn token_info(token: NativeToken) -> Token;
}

#[derive(Clone)]
pub enum NativeToken {
	GAFI,
	GAKI,
}

pub struct GafiCurrency {}

impl TokenInfo for GafiCurrency {
	fn token_info(token: NativeToken) -> Token {
		let gaki: Token = Token {
			id: 1,
			name: b"GAKI Token".to_vec(),
			symbol: "GAKI".as_bytes().to_vec(),
			decimals: 12,
		};
		let gafi: Token = Token {
			id: 2,
			name: b"GAFI Token".to_vec(),
			symbol: "GAFI".as_bytes().to_vec(),
			decimals: 10,
		};

		match token {
			NativeToken::GAKI => gaki,
			NativeToken::GAFI => gafi,
		}
	}
}

/// Express the native token as u128
///
/// # Examples
///
/// ```
/// use gafi_support::common::{NativeToken::GAKI, unit};
///
/// let balance = 10 * unit(GAKI);
/// assert_eq!(balance, 10_000_000_000_000_000_000);
/// ```
pub fn unit(token: NativeToken) -> u128 {
    10u128.saturating_pow(GafiCurrency::token_info(token).decimals.into())
}

/// 1 centi = 0.01 unit
pub fn centi(token: NativeToken) -> u128 {
    unit(token).saturating_div(100)
}

/// 1 milli = 0.001 unit
pub fn milli(token: NativeToken) -> u128 {
    unit(token).saturating_div(1000)
}

/// 1 millicent = 0.00001 unit
pub fn millicent(token: NativeToken) -> u128 {
    centi(token).saturating_div(1000)
}

/// 1 microcent = 0.000001 unit
pub fn microcent(token: NativeToken) -> u128 {
    milli(token).saturating_div(1000)
}

pub fn deposit(items: u32, bytes: u32, token: NativeToken) -> Balance {
    let mut balance = items as Balance;
    balance = balance
        .saturating_mul(20)
        .saturating_mul(unit(token.clone()))
        .saturating_add((bytes as Balance).saturating_mul(100).saturating_mul(millicent(token)));
    balance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit() {
        assert_eq!(unit(NativeToken::GAKI), 1_000_000_000_000_000_000);
    }

    #[test]
    fn test_centi() {
        assert_eq!(centi(NativeToken::GAKI), 10_000_000_000_000_000);
    }

    #[test]
    fn test_milli() {
        assert_eq!(milli(NativeToken::GAKI), 1_000_000_000_000_000);
    }

    #[test]
    fn test_millicent() {
        assert_eq!(millicent(NativeToken::GAKI), 10_000_000_000_000);
    }

    #[test]
    fn test_microcent() {
        assert_eq!(microcent(NativeToken::GAKI), 1_000_000_000_000);
    }

    #[test]
    fn test_deposit() {
        assert_eq!(deposit(5, 100, NativeToken::GAKI), 100_100_000_000_000_000_000);
    }
}