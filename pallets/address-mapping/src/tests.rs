use crate::{mock::*, Pallet, Error,
	 H160Mapping, AddressMapping, Id32Mapping, OriginAddressMapping};
use frame_support::{assert_err, assert_ok, traits::Currency};
use hex_literal::hex;
use sp_core::{H160};
use sp_runtime::{AccountId32};
use std::{str::FromStr};

#[test]
fn default_into_account_id_works() {
	ExtBuilder::default().build_and_execute(|| {
	let address: H160 = H160::from_str("b28049c6ee4f90ae804c70f860e55459e837e84b").unwrap();
	let account_id: AccountId32 = ProofAddressMapping::into_account_id(address);
	let origin_address: H160 = ProofAddressMapping::get_or_create_evm_address(account_id);
	assert_eq!(origin_address, address);

	let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
	let origin_address = ProofAddressMapping::get_evm_address(ALICE.clone());
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
		assert_eq!(ProofAddressMapping::verify_bond(ALICE, signature, address.to_fixed_bytes()), true, "verify should works");
	});
}

#[test]
fn bond_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let sender = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		{
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(&sender, 1_000_000);
			assert_eq!(Balances::free_balance(&sender), 1_000_000);
		}
		let origin_sender: H160 = ProofAddressMapping::get_or_create_evm_address(sender.clone());
		let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
		
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		let origin_address: AccountId32 = ProofAddressMapping::into_account_id(address);
		assert_ok!(ProofAddressMapping::bond(Origin::signed(sender.clone()), signature, address, false));

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
				ProofAddressMapping::bond(Origin::signed(ALICE), signature, address, true),
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
			ProofAddressMapping::bond(Origin::signed(BOB), signature, address, true),
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
			ProofAddressMapping::bond(Origin::signed(ALICE), signature, address, true),
			<Error<Test>>::SignatureOrAddressNotCorrect
		);
		}

		// insuffcient balance
		{
			run_to_block(10);
			let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
	
			let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
			let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
	
			assert_err!(ProofAddressMapping::bond(
				Origin::signed(ALICE.clone()),
				signature,
				address,
				false
			), pallet_balances::Error::<Test>::InsufficientBalance);
		}

		// rebond
		{
			run_to_block(10);
			let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
			let BOB  = AccountId32::from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();
			{
				let _ = pallet_balances::Pallet::<Test>::deposit_creating(&ALICE, 1_000_000);
				assert_eq!(Balances::free_balance(&ALICE), 1_000_000);

				let _ = pallet_balances::Pallet::<Test>::deposit_creating(&BOB, 1_000_000);
				assert_eq!(Balances::free_balance(&BOB), 1_000_000);
			}
			let address: H160 = H160::from_str("4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
			let alice_signature: [u8; 65] = hex!("0e3464d0b76371a158fc35ee5be6dd5989f34d6d7008331aa0415de1bf3bad6d6f4af7dbbd788b180f28d798c3dd3f1aa994e9057113424f1073256a3480ab651c");
			let bob_signature: [u8; 65] = hex!("e655cffe4ca3861c14dd36fe17fe2aaa37d4c2ea518dc27c788cfab1dfcb00c3065769c45d075218c881d1d39eaf72754c68dde05d2a5d80f271a8c67079d8a61c");
			
			assert_ok!(ProofAddressMapping::bond(
				Origin::signed(ALICE.clone()),
				alice_signature,
				address,
				false
			));

			assert_err!(
				ProofAddressMapping::bond(Origin::signed(ALICE.clone()), alice_signature, address, true),
				<Error<Test>>::AlreadyBond
			);

			assert_err!(
				ProofAddressMapping::bond(Origin::signed(BOB.clone()), bob_signature, address, true),
				<Error<Test>>::AlreadyBond
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
		let mapping_address = ProofAddressMapping::into_account_id(evm_address);
		{
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
			assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);

			let mapping_address_balance = EVM::account_basic(&evm_address).balance;
			assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
		}

		assert_ok!(ProofAddressMapping::transfer_all(mapping_address.clone(), ALICE.clone(), true));

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
		let mapping_address = ProofAddressMapping::into_account_id(evm_address);
		{
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
			assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);

			let mapping_address_balance = EVM::account_basic(&evm_address).balance;
			assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
		}

		assert_ok!(ProofAddressMapping::transfer_all(mapping_address.clone(), ALICE.clone(), false));
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
		let mapping_address= ProofAddressMapping::into_account_id(evm_address);
		{
			let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
			assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);

			let mapping_address_balance = EVM::account_basic(&evm_address).balance;
			assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
		}


		assert_ok!(ProofAddressMapping::bond(Origin::signed(ALICE.clone()), signature, evm_address, true));

		// evm_address balance should  equal to ALICE
		{
			assert_eq!(Balances::free_balance(&ALICE), EVM_BALANCE + ALICE_BALANCE - EXISTENTIAL_DEPOSIT - EXISTENTIAL_BOND_DEPOSIT);
			let mapping_address_balance = EVM::account_basic(&evm_address).balance;
			assert_eq!(mapping_address_balance, Balances::free_balance(&ALICE).into());
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
			
			let mapping_address =  ProofAddressMapping::into_account_id(evm_address);
			{
				let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
				assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);
	
				let mapping_address_balance = EVM::account_basic(&evm_address).balance;
				assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
			}
	
			assert_ok!(ProofAddressMapping::bond(Origin::signed(ALICE.clone()), signature, evm_address, true));
			
			let before_balance = Balances::free_balance(&ALICE);
			assert_ok!(ProofAddressMapping::unbond(Origin::signed(ALICE.clone())));
			let after_balance = Balances::free_balance(&ALICE);
			assert_eq!(before_balance, after_balance - EXISTENTIAL_BOND_DEPOSIT);
		}

		run_to_block(100);
		// address mapping should mapping back to original addresses
		let origin_id: AccountId32 = OriginAddressMapping::into_account_id(evm_address);
		let origin_address: H160 = ProofAddressMapping::get_or_create_evm_address(ALICE.clone());

		assert_eq!(H160Mapping::<Test>::get(evm_address), None);
		assert_eq!(Id32Mapping::<Test>::get(ALICE.clone()), None);
		
		assert_eq!(H160Mapping::<Test>::get(origin_address), None);
		assert_eq!(Id32Mapping::<Test>::get(origin_id.clone()), None);
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
			
			let mapping_address =  ProofAddressMapping::into_account_id(evm_address);
			{
				let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
				assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);
	
				let mapping_address_balance = EVM::account_basic(&evm_address).balance;
				assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
			}
	
			assert_ok!(ProofAddressMapping::bond(Origin::signed(ALICE.clone()), signature, evm_address, true));
		}

		let origin_id: AccountId32 = OriginAddressMapping::into_account_id(evm_address);
		let origin_address: H160 = ProofAddressMapping::get_or_create_evm_address(ALICE.clone());

		let mapping_account = ProofAddressMapping::into_account_id(evm_address);
		assert_eq!(mapping_account, ALICE);

		let mapping_sender = ProofAddressMapping::into_account_id(origin_address);
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
			
			let mapping_address =  ProofAddressMapping::into_account_id(evm_address);
			{
				let _ = pallet_balances::Pallet::<Test>::deposit_creating(&mapping_address, EVM_BALANCE);
				assert_eq!(Balances::free_balance(&mapping_address), EVM_BALANCE);
	
				let mapping_address_balance = EVM::account_basic(&evm_address).balance;
				assert_eq!(mapping_address_balance, (EVM_BALANCE - EXISTENTIAL_DEPOSIT).into());
			}
	
			assert_ok!(ProofAddressMapping::bond(Origin::signed(ALICE.clone()), signature, evm_address, true));
			assert_ok!(ProofAddressMapping::unbond(Origin::signed(ALICE.clone())));
		}

		let origin_id: AccountId32 = OriginAddressMapping::into_account_id(evm_address);
		let origin_address: H160 = ProofAddressMapping::get_or_create_evm_address(ALICE.clone());

		// let mapping_sender = ProofAddressMapping::into_account_id(origin_address);
		// assert_eq!(mapping_sender, ALICE);

		let mapping_account = ProofAddressMapping::into_account_id(evm_address);
		assert_eq!(mapping_account, origin_id);

	});
}

#[test]
fn unbond_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let ALICE = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
		assert_err!(ProofAddressMapping::unbond(Origin::signed(ALICE.clone())), <Error<Test>>::NonbondAccount);
	});
}