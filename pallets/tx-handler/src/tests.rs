/*
* This unittest should only test logic function e.g. Storage, Computation
* and not related with Currency e.g. Balances, Transaction Payment
*/
use crate::{mock::*, Error};
use frame_support::{assert_err, assert_ok};
use sp_core::H160;

use hex_literal::hex;
use std::str::FromStr;

#[test]
fn verify_owner_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);

		let sender = TEST_ACCOUNTS[0].0.clone();
		let account_bytes: [u8; 32] = sender.clone().into();
		let signature: [u8; 65];
		let message: [u8; 32];
		let address: H160 =  H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		// init value
		{
			let mut s = secp256k1::Secp256k1::new();
			let sk = hex!("5240c93f837385e95742426ebc0dc49bbbeded5a9aaec129ac9de1754ca98ccb");
			let sk = secp256k1::key::SecretKey::from_slice(&s, &sk).unwrap();
			let msg = sp_core::keccak_256(&account_bytes);
			message = msg;
			let msg = secp256k1::Message::from_slice(&msg).unwrap();
			let sig = s.sign_recoverable(&msg, &sk).unwrap();
			let (recover_id, sig_data) = sig.serialize_compact(&s);

			let mut sig: [u8; 65] = [0u8; 65];
			sig[0..64].copy_from_slice(&sig_data[..]);
			sig[64] = recover_id.to_i32() as u8;
			signature = sig;
		}
		assert_ok!(PalletTxHandler::verify_owner(sender, signature, message, address));
	});
}
