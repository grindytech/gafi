use crate::*;
// #[allow(unused)]
use crate::Pallet as GameCreator;
use crate::{Call, Config};
use frame_benchmarking::Box;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::log::info;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use pallet_evm::AddressMapping;
use pallet_evm::Runner;
use pallet_evm::{ExitReason, ExitSucceed};
use scale_info::prelude::format;
use scale_info::prelude::string::String;

use sp_core::{H160, U256};
use sp_std::str::FromStr;

fn make_free_balance<T: Config>(acc: &T::AccountId, balance: u64) {
    let balance_amount = balance.try_into().ok().unwrap();
    <T as pallet::Config>::Currency::make_free_balance_be(acc, balance_amount);
    <T as pallet::Config>::Currency::issue(balance_amount);
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn new_funded_account<T: Config>(index: u32, seed: u32, balance: u64) -> T::AccountId {
    let name: String = format!("{}{}", index, seed);
    let user = account(string_to_static_str(name), index, seed);
    make_free_balance::<T>(&user, balance);
    return user;
}


benchmarks! {
    claim_contract {
    	let s in 0 .. 1;
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let contract = H160::from_str("0xF0B9EaA0fAaC58d5d4F3224958D75a5370672231").unwrap();
        let sub_acc = T::AddressMapping::into_account_id(evm_acc);

        T::ContractCreator::insert_contract(&contract, &evm_acc);

        make_free_balance::<T>(&sub_acc, 1000_000_000_000_u64);

    }: _(RawOrigin::Signed(sub_acc), evm_acc)
}
