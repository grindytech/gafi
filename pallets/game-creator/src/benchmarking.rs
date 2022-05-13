use crate::*;
// #[allow(unused)]
use crate::Pallet as GameCreator;
use crate::{Call, Config};
use frame_benchmarking::Box;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::log::info;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use gafi_primitives::currency::{unit, NativeToken::GAKI};
use pallet_evm::AddressMapping;
use pallet_evm::Runner;
use pallet_evm::{ExitReason, ExitSucceed};
use scale_info::prelude::format;
use scale_info::prelude::string::String;

use sp_core::{H160, U256};
use sp_std::str::FromStr;

fn make_free_balance<T: Config>(acc: &T::AccountId, balance: u128) {
    let balance_amount = balance.try_into().ok().unwrap();
    <T as pallet::Config>::Currency::make_free_balance_be(acc, balance_amount);
    <T as pallet::Config>::Currency::issue(balance_amount);
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn new_funded_account<T: Config>(index: u32, seed: u32, balance: u128) -> T::AccountId {
    let name: String = format!("{}{}", index, seed);
    let user = account(string_to_static_str(name), index, seed);
    make_free_balance::<T>(&user, balance);
    return user;
}

benchmarks! {
    claim_contract {
        let s in 0 .. 1;
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = T::AddressMapping::into_account_id(evm_acc);
        make_free_balance::<T>(&sub_acc, 1_000 * unit(GAKI));
        
        let contract = H160::from_str("0xF0B9EaA0fAaC58d5d4F3224958D75a5370672231").unwrap();
        T::ContractCreator::insert_contract(&contract, &evm_acc);

    }: _(RawOrigin::Signed(sub_acc), contract)

    change_ownership {
        let s in 0 .. 1;
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = T::AddressMapping::into_account_id(evm_acc);
        make_free_balance::<T>(&sub_acc, 1_000 * unit(GAKI));
     
        let contract = H160::from_str("0xF0B9EaA0fAaC58d5d4F3224958D75a5370672231").unwrap();
        
        T::ContractCreator::insert_contract(&contract, &evm_acc);
        let new_owner = H160::from_str("0xD910E83396231988F79df2f1175a90e15d26aB71").unwrap();
        let new_owner = T::AddressMapping::into_account_id(new_owner);
        make_free_balance::<T>(&new_owner, 1_000 * unit(GAKI));

    }: _(RawOrigin::Signed(sub_acc), contract, new_owner)

    withdraw_contract {
        let s in 0 .. 1;
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = T::AddressMapping::into_account_id(evm_acc);
        make_free_balance::<T>(&sub_acc, 1_000 * unit(GAKI));
        
        let contract = H160::from_str("0xF0B9EaA0fAaC58d5d4F3224958D75a5370672231").unwrap();
        T::ContractCreator::insert_contract(&contract, &evm_acc);

        GameCreator::<T>::withdraw_contract(RawOrigin::Signed(sub_acc.clone()).into(), contract);

    }: _(RawOrigin::Signed(sub_acc), contract)
}
