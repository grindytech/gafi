use crate::{mock::*, Error, Pallet, ContractOwner};
use pallet_evm::ContractCreator;
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::currency::{unit, NativeToken::GAKI};
use pallet_evm::AddressMapping;
use sp_core::{H160, U256};
use sp_runtime::AccountId32;
use sp_std::str::FromStr;
use pallet_ethereum::RawOrigin;

fn make_deposit(account: &AccountId32, balance: u128) {
    let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(account: [u8; 32], balance: u128) -> AccountId32 {
    let acc: AccountId32 = AccountId32::from(account);
    make_deposit(&acc, balance);
    assert_eq!(Balances::free_balance(&acc), balance);
    return acc;
}


#[test]
fn claim_contract_works() {
    ExtBuilder::default().build_and_execute(|| {
        let contract = H160::from_str("0xF0B9EaA0fAaC58d5d4F3224958D75a5370672231").unwrap();
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = ProofAddressMapping::into_account_id(evm_acc);

        ContractCreator::<Test>::insert(contract, evm_acc);

        make_deposit(&sub_acc, 1_000 * unit(GAKI));
        assert_ok!(Pallet::<Test>::claim_contract(Origin::signed(sub_acc.clone()), contract));
        assert_eq!(ContractOwner::<Test>::get(contract), Some(sub_acc.clone()));
    })
}


#[test]
fn claim_contract_already_claim_fail() {
    ExtBuilder::default().build_and_execute(|| {
        let contract = H160::from_str("0xF0B9EaA0fAaC58d5d4F3224958D75a5370672231").unwrap();
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = ProofAddressMapping::into_account_id(evm_acc);

        ContractCreator::<Test>::insert(contract, evm_acc);

        make_deposit(&sub_acc, 1_000 * unit(GAKI));
        assert_ok!(Pallet::<Test>::claim_contract(Origin::signed(sub_acc.clone()), contract));

        assert_err!(Pallet::<Test>::claim_contract(Origin::signed(sub_acc.clone()), contract), Error::<Test>::ContractClaimed);
    })
}
