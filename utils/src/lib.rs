use secp256k1::{
	key::{SecretKey},
	Message, Secp256k1,
};
use sha3::{Digest, Keccak256, digest::crypto_common::Key};
use sp_core::{H160, H256, keccak_256};

pub fn recover_signer(sig: [u8; 65], msg: [u8; 32]) -> Option<H160>{
	let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg).ok()?;
	Some(H160::from(H256::from_slice(Keccak256::digest(&pubkey).as_slice())))
}



#[cfg(test)]
#[macro_use]
extern crate hex_literal;
use std::{fmt::Write, str::FromStr};

pub fn encode_hex(bytes: &[u8]) -> String {
	let mut s = String::with_capacity(bytes.len() * 2);
	for &b in bytes {
		write!(&mut s, "{:02x}", b).unwrap();
	}
	s
}

#[test]
fn verify_binding() {
	let mut s = Secp256k1::new();

	// privatekey only use for test
	let sk = hex!("5240c93f837385e95742426ebc0dc49bbbeded5a9aaec129ac9de1754ca98ccb");
	let evm_address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
	let sub_address: &[u8] = "5ETjddUFMJu8WjShUoB7a1149URVpVHtBDrijyXEaJhY9kZC".as_bytes();

	let sk = SecretKey::from_slice(&s, &sk).unwrap();

	let msg = keccak_256(sub_address);
	let message = Message::from_slice(&msg).unwrap();
	let sig = s.sign_recoverable(&message, &sk).unwrap();
	let (recover_id, sig_data) = sig.serialize_compact(&s);

	let mut signature: [u8; 65] = [0u8; 65]; 

	signature[0..64].copy_from_slice(&sig_data[..]);
	signature[64] = recover_id.to_i32() as u8;

	let address = recover_signer(signature, msg);
	assert_eq!(Some(evm_address), address, "recover signer address not correct");
}

