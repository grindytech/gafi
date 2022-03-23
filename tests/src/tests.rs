use crate::{eth_recover, EcdsaSignature, to_ascii_hex};
use parity_scale_codec::Encode;
use hex_literal::hex;
use serde::Serialize;

#[test]
fn simple_eth_sig_works() {
	// "Pay RUSTs to the TEST account:2a00000000000000"
	let sig: [u8; 65] = hex!["4119b0714c19ed692f0ae497dc78ba43207e39a93434797950a03581e16a7de042a398132f115acb9c9b798a9e30b074fc2bd62ad5ba2eeb2ca70585efdc70531b"];
	let sig = EcdsaSignature(sig);
	let who = 42u64.using_encoded(to_ascii_hex);
	let address = std::str::from_utf8(&who);
	println!("address: {:?}", address);
	let signer = eth_recover(&sig, &who, &[][..]).unwrap();

	assert_eq!(signer.0, hex!["b28049C6EE4F90AE804C70F860e55459E837E84b"]);
}


#[test]
fn real_eth_sig_works() {
	// "Pay RUSTs to the TEST account:d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
	let sig: [u8; 65] = hex!["72c830ea68d77630c31392a6a2555849e79ab966f1f49b470f17c7d5fa4b294f61d2a3d81595d2f6a2ebc8b9df01566ba9505fc82164f2d4d2dffb147a0826251b"];
	let sig = EcdsaSignature(sig);
	let who = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".using_encoded(to_ascii_hex);
	let address = std::str::from_utf8(&who);
	println!("address: {:?}", address);
	let signer = eth_recover(&sig, &who, &[][..]).unwrap();

	assert_eq!(signer.0, hex!["b28049C6EE4F90AE804C70F860e55459E837E84b"]);
}