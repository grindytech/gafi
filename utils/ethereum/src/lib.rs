/*
	re-use functions from https://github.com/paritytech/polkadot/blob/master/runtime/common/src/claims.rs
*/

#![cfg_attr(not(feature = "std"), no_std)]

use sha3::{Digest, Keccak256};
use sp_core::{H160, H256};

pub fn recover_signer(sig: [u8; 65], msg: [u8; 32]) -> Option<H160>{
	let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg).ok()?;
	Some(H160::from(H256::from_slice(Keccak256::digest(&pubkey).as_slice())))
}

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_runtime::{
	RuntimeDebug,
};
use sp_std::{prelude::*};

#[cfg(test)]
mod tests;

/// An Ethereum address (i.e. 20 bytes, used to represent an Ethereum account).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug,  TypeInfo)]
pub struct EthereumAddress(pub [u8; 20]);

#[cfg(feature = "std")]
impl Serialize for EthereumAddress {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let hex: String = rustc_hex::ToHex::to_hex(&self.0[..]);
		serializer.serialize_str(&format!("0x{}", hex))
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for EthereumAddress {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let base_string = String::deserialize(deserializer)?;
		let offset = if base_string.starts_with("0x") { 2 } else { 0 };
		let s = &base_string[offset..];
		if s.len() != 40 {
			return Err(serde::de::Error::custom(
				"Bad length of Ethereum address (should be 42 including '0x')",
			));
		}
		let raw: Vec<u8> = rustc_hex::FromHex::from_hex(s)
			.map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?;
		let mut r = Self::default();
		r.0.copy_from_slice(&raw);
		Ok(r)
	}
}

#[derive(Encode, Decode, Clone, TypeInfo)]
pub struct EcdsaSignature(pub [u8; 65]);

impl PartialEq for EcdsaSignature {
	fn eq(&self, other: &Self) -> bool {
		&self.0[..] == &other.0[..]
	}
}

impl sp_std::fmt::Debug for EcdsaSignature {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		write!(f, "EcdsaSignature({:?})", &self.0[..])
	}
}

// Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign` would sign.
fn ethereum_signable_message(what: &[u8], extra: &[u8], prefix: &[u8]) -> Vec<u8> {
    let mut l = prefix.len() + what.len() + extra.len();
    let mut rev = Vec::new();
    while l > 0 {
        rev.push(b'0' + (l % 10) as u8);
        l /= 10;
    }
    let mut v = b"\x19Ethereum Signed Message:\n".to_vec();
    v.extend(rev.into_iter().rev());
    v.extend_from_slice(&prefix[..]);
    v.extend_from_slice(what);
    v.extend_from_slice(extra);
    v
}

// Attempts to recover the Ethereum address from a message signature signed by using
// the Ethereum RPC's `personal_sign` and `eth_sign`.
pub fn eth_recover(s: &EcdsaSignature, what: &[u8], extra: &[u8], prefix: &[u8]) -> Option<EthereumAddress> {
    let msg = keccak_256(&ethereum_signable_message(what, extra, prefix));
    let mut res = EthereumAddress::default();
    res.0
        .copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
    Some(res)
}

/// Converts the given binary data into ASCII-encoded hex. It will be twice the length.
pub fn to_ascii_hex(data: &[u8]) -> Vec<u8> {
	let mut r = Vec::with_capacity(data.len() * 2);
	let mut push_nibble = |n| r.push(if n < 10 { b'0' + n } else { b'a' - 10 + n });
	for &b in data.iter() {
		push_nibble(b / 16);
		push_nibble(b % 16);
	}
	r
}