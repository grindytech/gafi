
use crate::{mock::*, Pallet, ContractOwned, Error, ContractMapping};
use frame_support::{assert_err, assert_ok, traits::Currency};
use sp_runtime::AccountId32;
use sp_core::H160;
use sp_std::str::FromStr;
use proof_address_mapping::ProofAddressMapping;
use pallet_evm::AddressMapping;

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
fn claim_reward_works() {

    ExtBuilder::default().build_and_execute(|| {
        let owner_balance = 1_000_000_000_000_000_000_000_u128;
        let owner = new_account([0_u8; 32], owner_balance);
    
        let contract_balance = 10_000_000_000_000_000_000_u128;
        let contract_addr: H160 = H160::from_str("0x0A6617b82B594C83240092BDc86E2e16354d1456").unwrap();
        let sub_contract_addr: AccountId32 = ProofAddressMapping::<Test>::into_account_id(contract_addr);
        make_deposit(&sub_contract_addr, contract_balance);

        ContractOwned::<Test>::insert(contract_addr, owner.clone());

        assert_ok!(GameCreator::claim_reward(Origin::signed(owner.clone()), contract_addr));

        assert_eq!(Balances::free_balance(&owner), owner_balance + contract_balance);
        assert_eq!(Balances::free_balance(&sub_contract_addr), 0_u128);
    })
}

#[test]
fn mapping_contract_works() {
    ExtBuilder::default().build_and_execute(|| {
        let contract: H160 = H160::from_str("0xB5C35Cbc5c46DC34A5A89E7cD73B2b3A1e1DC008").unwrap();
        let owner: H160 = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();

        GameCreator::mapping_contract(&contract, &owner);

        assert_eq!(ContractMapping::<Test>::get(contract.clone()), Some(owner.clone()));
    })
}