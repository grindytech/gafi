
use hex_literal::hex;
use sp_core::H160;
use std::str::FromStr;
use crate::{recover_signer};

#[test]
fn verify_binding() {
	let mut s = secp256k1::Secp256k1::new();

	// privatekey only use for test
	let sk = hex!("5240c93f837385e95742426ebc0dc49bbbeded5a9aaec129ac9de1754ca98ccb");
	let evm_address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
	let sub_address: &[u8] = "5ETjddUFMJu8WjShUoB7a1149URVpVHtBDrijyXEaJhY9kZC".as_bytes();

	let sk = secp256k1::key::SecretKey::from_slice(&s, &sk).unwrap();

	let msg = sp_core::keccak_256(sub_address);
	let message = secp256k1::Message::from_slice(&msg).unwrap();
	let sig = s.sign_recoverable(&message, &sk).unwrap();
	let (recover_id, sig_data) = sig.serialize_compact(&s);

	let mut signature: [u8; 65] = [0u8; 65]; 

	signature[0..64].copy_from_slice(&sig_data[..]);
	signature[64] = recover_id.to_i32() as u8;

	let address = recover_signer(signature, msg);
	assert_eq!(Some(evm_address), address, "recover signer address not correct");
}

