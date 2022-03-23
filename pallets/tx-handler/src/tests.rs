/*
* This unittest should only test logic function e.g. Storage, Computation
* and not related with Currency e.g. Balances, Transaction Payment
*/
use crate::{mock::*, Error, Mapping};
use frame_support::{assert_err, assert_ok};
use hex_literal::hex;
use parity_scale_codec::{Decode, Encode};
use sp_core::H160;
use sp_runtime::AccountId32;
use std::str::FromStr;

#[test]
fn verify_owner_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let signature: [u8; 65] = hex!("2bda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		assert_ok!(PalletTxHandler::verify_bind(ALICE, signature, address.to_fixed_bytes()));

	});
}

#[test]
fn bind_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();

		let signature: [u8; 65] = hex!("2bda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		assert_ok!(PalletTxHandler::bind(Origin::signed(ALICE.clone()), signature, address));

		let account_id = Mapping::<Test>::get(address).unwrap();
		assert_eq!(account_id, ALICE, "AccountId not correct");
	});
}

#[test]
fn bind_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		// incorrect address
		{
			run_to_block(10);
			let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
			let signature: [u8; 65] = hex!("2bda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
			let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84c").unwrap(); //incorrect address

			assert_err!(
				PalletTxHandler::bind(Origin::signed(ALICE), signature, address),
				<Error<Test>>::SignatureOrAddressNotCorrect
			);
		}

		// incorrect sender
		{
			run_to_block(10);
		let BOB = AccountId32::from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();
		let signature: [u8; 65] = hex!("2bda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();

		assert_err!(
			PalletTxHandler::bind(Origin::signed(BOB), signature, address),
			<Error<Test>>::SignatureOrAddressNotCorrect
		);
		}

		// incorrect signature
		{
			run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();

		let signature: [u8; 65] = hex!("2cda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		assert_err!(
			PalletTxHandler::bind(Origin::signed(ALICE), signature, address),
			<Error<Test>>::SignatureOrAddressNotCorrect
		);
		}

		// account already bind
		{
			run_to_block(10);
			let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
	
			let signature: [u8; 65] = hex!("2bda6694b9b24c4dfd0bd6ae39e82cb20ce9c4726e5b84e677a460bfb402ae5f0a3cfb1fa0967aa6cbc02cbc3140442075be0152473d845ee5316df56127be1c1b");
			let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
	
			assert_ok!(PalletTxHandler::bind(
				Origin::signed(ALICE.clone()),
				signature,
				address
			));

			assert_err!(
				PalletTxHandler::bind(Origin::signed(ALICE.clone()), signature, address),
				<Error<Test>>::AccountAlreadyBind
			);
		}
	})
}
