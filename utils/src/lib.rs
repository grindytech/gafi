#![cfg_attr(not(feature = "std"), no_std)]

use sha3::{Digest, Keccak256};
use sp_core::{H160, H256};

#[cfg(test)]
mod tests;

pub fn recover_signer(sig: [u8; 65], msg: [u8; 32]) -> Option<H160>{
	let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg).ok()?;
	Some(H160::from(H256::from_slice(Keccak256::digest(&pubkey).as_slice())))
}
