/*
* This unittest should only test logic function e.g. Storage, Computation
* and not related with Currency e.g. Balances, Transaction Payment
*/
use crate::{mock::*, Error, Mapping};
use frame_support::{assert_err, assert_ok};
use sp_core::H160;

use hex_literal::hex;
use sp_runtime::AccountId32;
use std::str::FromStr;

#[test]
fn verify_owner_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);

		let sender = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let account_bytes: [u8; 32] = sender.clone().into();
		let signature: [u8; 65];
		let message: [u8; 32];
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
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
		assert_ok!(PalletTxHandler::verify_owner(sender.clone(), signature, address));
	});
}

#[test]
fn bind_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);

		let sender = TEST_ACCOUNTS[0].0.clone();
		let account_bytes: [u8; 32] = sender.clone().into();
		let signature: [u8; 65];
		let message: [u8; 32];
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
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
		assert_ok!(PalletTxHandler::bind(
			Origin::signed(sender.clone()),
			signature,
			address
		));

		let account_id = Mapping::<Test>::get(address).unwrap();
		assert_eq!(account_id, sender, "AccountId not correct");
	});
}

#[test]
fn bind_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		// incorrect address
		{
			let sender = TEST_ACCOUNTS[0].0.clone();
			let account_bytes: [u8; 32] = sender.clone().into();
			let signature: [u8; 65];
			let message: [u8; 32];
			let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84c").unwrap(); //change to incorrect address
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
			assert_err!(
				PalletTxHandler::bind(Origin::signed(sender.clone()), signature, address),
				<Error<Test>>::AddressNotCorrect
			);
		}

		// incorrect sender
		{
			let sender = TEST_ACCOUNTS[0].0.clone();
			let account_bytes: [u8; 32] = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap().into();
			let signature: [u8; 65];
			let message: [u8; 32];
			let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
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
			assert_err!(
				PalletTxHandler::bind(Origin::signed(sender.clone()), signature, address),
				<Error<Test>>::AddressNotCorrect
			);
		}

		// incorrect signature
		{
			let sender = TEST_ACCOUNTS[0].0.clone();
			let account_bytes: [u8; 32] = sender.clone().into();
			let signature: [u8; 65];
			let message: [u8; 32];
			let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
			// init value
			{
				let msg = sp_core::keccak_256(&account_bytes);
				message = msg;
				let mut sig: [u8; 65] = [0u8; 65];
				signature = sig;
			}
			assert_err!(
				PalletTxHandler::bind(Origin::signed(sender.clone()), signature, address),
				<Error<Test>>::CanNotRecoverSigner
			);
		}

		// account already bind
		{
			let sender = TEST_ACCOUNTS[0].0.clone();
			let account_bytes: [u8; 32] = sender.clone().into();
			let signature: [u8; 65];
			let message: [u8; 32];
			let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
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

			assert_ok!(PalletTxHandler::bind(
				Origin::signed(sender.clone()),
				signature,
				address
			));

			assert_err!(
				PalletTxHandler::bind(Origin::signed(sender.clone()), signature, address),
				<Error<Test>>::AccountAlreadyBind
			);
		}
	})
}
