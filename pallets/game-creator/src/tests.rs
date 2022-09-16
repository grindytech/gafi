use crate::{mock::*, ContractOwner, Error, Pallet};
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::currency::{unit, NativeToken::GAKI};
use pallet_evm::AddressMapping;
use pallet_evm::{ExitReason, ExitSucceed, Runner};
use sp_core::{
    bytes::{from_hex},
    H160, U256,
};
use sp_runtime::AccountId32;
use sp_std::str::FromStr;

fn make_deposit(account: &AccountId32, balance: u128) {
    let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn _new_account(account: [u8; 32], balance: u128) -> AccountId32 {
    let acc: AccountId32 = AccountId32::from(account);
    make_deposit(&acc, balance);
    assert_eq!(Balances::free_balance(&acc), balance);
    return acc;
}

fn deploy_contract(caller: H160) -> H160 {
    let contract = from_hex(
		"0x608060405234801561001057600080fd5b5060b88061001f6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063165c4a1614602d575b600080fd5b606060048036036040811015604157600080fd5b8101908080359060200190929190803590602001909291905050506076565b6040518082815260200191505060405180910390f35b600081830290509291505056fea265627a7a723158201f3db7301354b88b310868daf4395a6ab6cd42d16b1d8e68cdf4fdd9d34fffbf64736f6c63430005110032"
	).unwrap();
    let result = <Test as pallet_evm::Config>::Runner::create(
        caller,
        contract,
        U256::from(0_u64),
        1000000,
        None,
        None,
        None,
        vec![],
        false,
        false,
        <Test as pallet_evm::Config>::config(),
    )
    .unwrap();
    assert_eq!(
        result.exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );

    result.value
}

#[test]
fn claim_contract_works() {
    ExtBuilder::default().build_and_execute(|| {
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = ProofAddressMapping::into_account_id(evm_acc);
        let sub_acc_balance = 1_000 * unit(GAKI);
        let contract_address = deploy_contract(evm_acc);

        make_deposit(&sub_acc, sub_acc_balance);
        assert_ok!(Pallet::<Test>::claim_contract(
            Origin::signed(sub_acc.clone()),
            contract_address
        ));
        assert_eq!(
            ContractOwner::<Test>::get(contract_address).unwrap().0,
            sub_acc.clone()
        );

        assert_eq!(Balances::free_balance(sub_acc.clone()), sub_acc_balance - GAME_CREATE_FEE);
    })
}

#[test]
fn claim_contract_already_claim_fail() {
    ExtBuilder::default().build_and_execute(|| {
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = ProofAddressMapping::into_account_id(evm_acc);
        make_deposit(&sub_acc, 1_000 * unit(GAKI));
        let contract_address = deploy_contract(evm_acc);
        assert_ok!(Pallet::<Test>::claim_contract(
            Origin::signed(sub_acc.clone()),
            contract_address
        ));

        assert_err!(
            Pallet::<Test>::claim_contract(Origin::signed(sub_acc.clone()), contract_address),
            Error::<Test>::ContractClaimed
        );
    })
}

#[test]
fn claim_contract_not_owner_fail() {
    ExtBuilder::default().build_and_execute(|| {
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = ProofAddressMapping::into_account_id(evm_acc);
        make_deposit(&sub_acc, 1_000 * unit(GAKI));
        let contract_address = deploy_contract(evm_acc);
        assert_err!(
            Pallet::<Test>::claim_contract(
                Origin::signed(AccountId32::from([0u8; 32])),
                contract_address
            ),
            Error::<Test>::NotContractOwner
        );
    })
}

#[test]
fn change_ownership_works() {
    ExtBuilder::default().build_and_execute(|| {
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = ProofAddressMapping::into_account_id(evm_acc);
        let sub_acc_balance = 1_000 * unit(GAKI);
        let contract_address = deploy_contract(evm_acc);

        make_deposit(&sub_acc, sub_acc_balance);
        assert_ok!(Pallet::<Test>::claim_contract(
            Origin::signed(sub_acc.clone()),
            contract_address
        ));

        let new_owner = AccountId32::from([0u8; 32]);
        make_deposit(&new_owner, 1_000 * unit(GAKI));

        assert_ok!(Pallet::<Test>::change_ownership(
            Origin::signed(sub_acc.clone()),
            contract_address,
            new_owner.clone()
        ));

        assert_eq!(
            ContractOwner::<Test>::get(contract_address).unwrap().0,
            new_owner.clone()
        );

        assert_eq!(Balances::free_balance(&sub_acc), sub_acc_balance - GAME_CREATE_FEE);
    })
}

#[test]
fn change_ownership_not_owner_fail() {
    ExtBuilder::default().build_and_execute(|| {
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = ProofAddressMapping::into_account_id(evm_acc);

        let contract_address = deploy_contract(evm_acc);

        make_deposit(&sub_acc, 1_000 * unit(GAKI));
        assert_ok!(Pallet::<Test>::claim_contract(
            Origin::signed(sub_acc.clone()),
            contract_address
        ));

        let new_owner = AccountId32::from([0u8; 32]);
        make_deposit(&new_owner, 1_000 * unit(GAKI));

        assert_err!(
            Pallet::<Test>::change_ownership(
                Origin::signed(AccountId32::from([1u8; 32])),
                contract_address,
                new_owner.clone()
            ),
            Error::<Test>::NotContractOwner
        );
    })
}

#[test]
fn withdraw_contract_works() {
    ExtBuilder::default().build_and_execute(|| {
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = ProofAddressMapping::into_account_id(evm_acc);
        let sub_acc_balance = 1_000 * unit(GAKI);
        let contract_address = deploy_contract(evm_acc);

        make_deposit(&sub_acc, sub_acc_balance);
        assert_ok!(Pallet::<Test>::claim_contract(
            Origin::signed(sub_acc.clone()),
            contract_address
        ));

        assert_ok!(Pallet::<Test>::withdraw_contract(Origin::signed(sub_acc.clone()), contract_address));

        assert_eq!(ContractOwner::<Test>::get(contract_address), None);
        assert_eq!(Balances::free_balance(sub_acc.clone()), sub_acc_balance);
    })
}

#[test]
fn withdraw_afer_change_ownership_works() {
    ExtBuilder::default().build_and_execute(|| {
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = ProofAddressMapping::into_account_id(evm_acc);
        let sub_acc_balance = 1_000 * unit(GAKI);
        let contract_address = deploy_contract(evm_acc);

        make_deposit(&sub_acc, sub_acc_balance);
        assert_ok!(Pallet::<Test>::claim_contract(
            Origin::signed(sub_acc.clone()),
            contract_address
        ));

        let new_owner = AccountId32::from([0u8; 32]);
        let new_owner_balance = 100 * unit(GAKI); 
        make_deposit(&new_owner, new_owner_balance);

        assert_ok!(Pallet::<Test>::change_ownership(
            Origin::signed(sub_acc.clone()),
            contract_address,
            new_owner.clone()
        ));

        assert_ok!(Pallet::<Test>::withdraw_contract(Origin::signed(new_owner.clone()), contract_address));

        assert_eq!(Balances::free_balance(sub_acc.clone()), sub_acc_balance - GAME_CREATE_FEE);
        assert_eq!(Balances::free_balance(new_owner.clone()), new_owner_balance + GAME_CREATE_FEE);

    })
}
