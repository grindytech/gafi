use crate::{mock::*, Error, H160Mapping, Config, AddressMapping, pallet, ProofAddressMapping, OriginAddressMapping, Id32Mapping};
use frame_support::{assert_err, assert_ok, traits::Currency};
use hex_literal::hex;
use sp_core::{H160};
use sp_runtime::{AccountId32};
use std::{str::FromStr};

#[test]
fn default_into_account_id_works() {
	ExtBuilder::default().build_and_execute(|| {
	let address: H160 = H160::from_str("b28049c6ee4f90ae804c70f860e55459e837e84b").unwrap();
	let account_id: AccountId32 = ProofAddressMapping::<Test>::into_account_id(address);
	let origin_address: H160 = PalletTxHandler::get_or_create_evm_address(account_id);
	assert_eq!(origin_address, address);

	let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
	let origin_address = PalletTxHandler::get_evm_address(ALICE.clone());
	assert_eq!(origin_address, None);

	});
}

#[test]
fn verify_owner_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		assert_eq!(PalletTxHandler::verify_bond(ALICE, signature, address.to_fixed_bytes()), true, "verify should works");
	});
}

#[test]
fn bond_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let sender = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let origin_sender: H160 = PalletTxHandler::get_or_create_evm_address(sender.clone());
		let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
		
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		let origin_address: AccountId32 = ProofAddressMapping::<Test>::into_account_id(address);
		assert_ok!(PalletTxHandler::bond(Origin::signed(sender.clone()), signature, address, false));

		assert_eq!(H160Mapping::<Test>::get(address), Some(sender.clone()));
		assert_eq!(Id32Mapping::<Test>::get(sender), Some(address));

		assert_eq!(H160Mapping::<Test>::get(origin_sender), Some(origin_address.clone()));
		assert_eq!(Id32Mapping::<Test>::get(origin_address), Some(origin_sender));

	});
}

#[test]
fn bond_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		// incorrect address
		{
			run_to_block(10);
			let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
			let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
			let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84c").unwrap(); //incorrect address

			assert_err!(
				PalletTxHandler::bond(Origin::signed(ALICE), signature, address, true),
				<Error<Test>>::SignatureOrAddressNotCorrect
			);
		}

		// incorrect sender
		{
			run_to_block(10);
		let BOB = AccountId32::from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();
		let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();

		assert_err!(
			PalletTxHandler::bond(Origin::signed(BOB), signature, address, true),
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
			PalletTxHandler::bond(Origin::signed(ALICE), signature, address, true),
			<Error<Test>>::SignatureOrAddressNotCorrect
		);
		}

		// account already bond
		{
			run_to_block(10);
			let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
	
			let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
			let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
	
			assert_ok!(PalletTxHandler::bond(
				Origin::signed(ALICE.clone()),
				signature,
				address,
				false
			));

			assert_err!(
				PalletTxHandler::bond(Origin::signed(ALICE.clone()), signature, address, true),
				<Error<Test>>::EVMAccountAlreadyBond
			);
		}
	})
}


#[test]
fn transfer_all_keep_alive_works() {
		ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		const EVM_BALANCE: u64 = 1_000_000_000;
		const ALICE_BALANCE: u64 = 1_000_000_000;
		let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		{
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
			assert_eq!(Balances::free_balance(&ALICE), ALICE_BALANCE);
		}
		
		let evm_address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		let mapping_address = <Test as Config>::AddressMapping::into_account_id(evm_address);
		{
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
			assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);

			let mapping_address_balance = EVM::account_basic(&evm_address).balance;
			assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
		}

		assert_ok!(PalletTxHandler::transfer_all(evm_address, ALICE.clone(), true));

		// evm_address balance should  
		{
			assert_eq!(Balances::free_balance(&mapping_address), EXISTENTIAL_DEPOSIT);
			assert_eq!(Balances::free_balance(&ALICE), EVM_BALANCE + ALICE_BALANCE - EXISTENTIAL_DEPOSIT);
		}
	});
}


#[test]
fn transfer_all_allow_death_works() {
		ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		const EVM_BALANCE: u64 = 1_000_000_000;
		const ALICE_BALANCE: u64 = 1_000_000_000;
		let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		{
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
			assert_eq!(Balances::free_balance(&ALICE), ALICE_BALANCE);
		}
		
		let evm_address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		let mapping_address = <Test as Config>::AddressMapping::into_account_id(evm_address);
		{
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
			assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);

			let mapping_address_balance = EVM::account_basic(&evm_address).balance;
			assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
		}

		assert_ok!(PalletTxHandler::transfer_all(evm_address, ALICE.clone(), false));
		// evm_address balance should  
		{
			assert_eq!(Balances::free_balance(&mapping_address), 0);
			assert_eq!(Balances::free_balance(&ALICE), EVM_BALANCE + ALICE_BALANCE);
		}
		
	});
}


#[test]
fn bond_account_balances() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		const EVM_BALANCE: u64 = 1_000_000_000;
		const ALICE_BALANCE: u64 = 1_000_000_000;
		let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		{
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
			assert_eq!(Balances::free_balance(&ALICE), ALICE_BALANCE);
		}
		
		let evm_address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		let mapping_address = <Test as Config>::AddressMapping::into_account_id(evm_address);
		{
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
			assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);

			let mapping_address_balance = EVM::account_basic(&evm_address).balance;
			assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
		}


		assert_ok!(PalletTxHandler::bond(Origin::signed(ALICE.clone()), signature, evm_address, true));

		// evm_address balance should  
		{
			assert_eq!(Balances::free_balance(&ALICE), EVM_BALANCE + ALICE_BALANCE - EXISTENTIAL_DEPOSIT);
			let mapping_address_balance = EVM::account_basic(&evm_address).balance;
			assert_eq!(mapping_address_balance, (EVM_BALANCE + ALICE_BALANCE - 2*EXISTENTIAL_DEPOSIT).into());
		}
	});
}

#[test]
fn unbond_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let evm_address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
	
		{
			const EVM_BALANCE: u64 = 1_000_000_000;
			const ALICE_BALANCE: u64 = 1_000_000_000;
			let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
			{
				let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
				assert_eq!(Balances::free_balance(&ALICE), ALICE_BALANCE);
			}
			
			let mapping_address = <Test as Config>::AddressMapping::into_account_id(evm_address);
			{
				let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
				assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);
	
				let mapping_address_balance = EVM::account_basic(&evm_address).balance;
				assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
			}
	
			assert_ok!(PalletTxHandler::bond(Origin::signed(ALICE.clone()), signature, evm_address, true));
			assert_ok!(PalletTxHandler::unbond(Origin::signed(ALICE.clone())));
		}

		run_to_block(100);
		// address mapping should mapping back to original addresses
		let origin_id: AccountId32 = OriginAddressMapping::into_account_id(evm_address);
		let origin_address: H160 = PalletTxHandler::get_or_create_evm_address(ALICE.clone());

		assert_eq!(H160Mapping::<Test>::get(evm_address), None);
		assert_eq!(Id32Mapping::<Test>::get(ALICE.clone()), None);
		
		assert_eq!(H160Mapping::<Test>::get(origin_address), None);
		assert_eq!(Id32Mapping::<Test>::get(origin_id.clone()), None);
		
	
		// let mapping_address: AccountId32 = ProofAddressMapping::<Test>::into_account_id(evm_address);
		// assert_eq!(mapping_address, origin_id);
		
		// let mapping_account: AccountId32 = ProofAddressMapping::<Test>::into_account_id(origin_address);
		// assert_eq!(mapping_account, ALICE.clone());

	});
}

#[test]
fn proof_address_mapping_when_bond_works() {
	ExtBuilder::default().build_and_execute(|| {
	run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let evm_address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
	
		{
			const EVM_BALANCE: u64 = 1_000_000_000;
			const ALICE_BALANCE: u64 = 1_000_000_000;
			let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
			{
				let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
				assert_eq!(Balances::free_balance(&ALICE), ALICE_BALANCE);
			}
			
			let mapping_address = <Test as Config>::AddressMapping::into_account_id(evm_address);
			{
				let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
				assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);
	
				let mapping_address_balance = EVM::account_basic(&evm_address).balance;
				assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
			}
	
			assert_ok!(PalletTxHandler::bond(Origin::signed(ALICE.clone()), signature, evm_address, true));
		}

		let origin_id: AccountId32 = OriginAddressMapping::into_account_id(evm_address);
		let origin_address: H160 = PalletTxHandler::get_or_create_evm_address(ALICE.clone());

		let mapping_account = ProofAddressMapping::<Test>::into_account_id(evm_address);
		assert_eq!(mapping_account, ALICE);

		let mapping_sender = ProofAddressMapping::<Test>::into_account_id(origin_address);
		assert_eq!(mapping_sender, origin_id);
	});
}

#[test]
fn proof_address_mapping_when_unbond_works() {
	ExtBuilder::default().build_and_execute(|| {
	run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		let evm_address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
	
		{
			const EVM_BALANCE: u64 = 1_000_000_000;
			const ALICE_BALANCE: u64 = 1_000_000_000;
			let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
			{
				let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, ALICE_BALANCE);
				assert_eq!(Balances::free_balance(&ALICE), ALICE_BALANCE);
			}
			
			let mapping_address = <Test as Config>::AddressMapping::into_account_id(evm_address);
			{
				let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
				assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);
	
				let mapping_address_balance = EVM::account_basic(&evm_address).balance;
				assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
			}
	
			assert_ok!(PalletTxHandler::bond(Origin::signed(ALICE.clone()), signature, evm_address, true));
			assert_ok!(PalletTxHandler::unbond(Origin::signed(ALICE.clone())));
		}

		let origin_id: AccountId32 = OriginAddressMapping::into_account_id(evm_address);
		let origin_address: H160 = PalletTxHandler::get_or_create_evm_address(ALICE.clone());

		// let mapping_sender = ProofAddressMapping::<Test>::into_account_id(origin_address);
		// assert_eq!(mapping_sender, ALICE);

		let mapping_account = ProofAddressMapping::<Test>::into_account_id(evm_address);
		assert_eq!(mapping_account, origin_id);

	});
}

#[test]
fn unbond_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		assert_err!(PalletTxHandler::unbond(Origin::signed(ALICE.clone())), <Error<Test>>::NonbondAccount);
	});
}