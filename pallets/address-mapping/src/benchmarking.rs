//! Benchmarking setup for gafi-tx

use super::*;
#[allow(unused)]
use crate::Pallet as Mapping;
use crate::{Call, Config};
use frame_benchmarking::Box;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use hex_literal::hex;
use scale_info::prelude::format;
use scale_info::prelude::string::String;
use sp_core::H160;
use frame_support::log::info;
use sp_std::{str::FromStr};
use pallet_evm::AddressMapping;
use gu_convertor::into_account;

fn get_signature(index: u32) -> [u8; 65] {
	let signatures: [[u8; 65]; 2] = [
        hex!("2bf57eba60c4b36b2f040e28d9be64410d1846f899dea9d255be27e69b0ff33b41faa52bebd1f0b66e5d1efe89f162733a2ab19dea008ddd5d0e38e64532c4461c"),
        hex!("28a8a9d3fc8f1da0c039b451c2b571ea3a76b79d420a02f4700deebcac25fcd6422a3d9e4aca5d91cae28a851d62ba4f31b546118113fe304bae02274b02efac1b"),
    ];

	return signatures[index as usize];
}

fn get_address(index: u32) -> H160 {
    let addresses = [
        H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap(),
        H160::from_str("427491884a4baCA9a9a337e919f3aC96A0b88E64").unwrap(),
    ];
	return addresses[index as usize];
}

fn get_withdraw(index: u32) -> bool {
	let options = [true, true];
	return options[index as usize];
}

fn string_to_static_str(s: String) -> &'static str {
	Box::leak(s.into_boxed_str())
}

fn new_funded_account<T: Config>(index: u32, seed: u32, amount: u64) -> T::AccountId {
	let balance_amount = amount.try_into().ok().unwrap();
	let name: String = format!("{}{}", index, seed);
	let user = account(string_to_static_str(name), index, seed);
	<T as pallet::Config>::Currency::make_free_balance_be(&user, balance_amount);
	<T as pallet::Config>::Currency::issue(balance_amount);
	return user;
}

fn init_funded_h160<T: Config>(address: H160) {
	let account_id = Pallet::<T>::into_account_id(address);
	let account = into_account::<T::AccountId>(account_id.into()).unwrap();
	let balance_amount = 1000_000_000u64.try_into().ok().unwrap();
	<T as pallet::Config>::Currency::make_free_balance_be(&account, balance_amount);
	<T as pallet::Config>::Currency::issue(balance_amount);
}

benchmarks! {
	where_clause { where sp_runtime::AccountId32: From<<T as frame_system::Config>::AccountId>,
		[u8; 32]: From<<T as frame_system::Config>::AccountId>,
	 }
	bond {
		let s in 0 .. 1;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u64);

        let who = caller.using_encoded(to_ascii_hex);
	    let address = String::from_utf8(who);

        info!("address: {:?}", address);

		let signature: [u8; 65] = get_signature(s);
		let address: H160 = get_address(s);
		let withdraw = get_withdraw(s);
	}: _(RawOrigin::Signed(caller), signature, address, withdraw)

	unbond {
		let s in 0 .. 1;
		let caller = new_funded_account::<T>(s, s, 1000_000_000u64);
		let signature: [u8; 65] = get_signature(s);
		let address: H160 = get_address(s);
		init_funded_h160::<T>(address);
		let withdraw = get_withdraw(s);
		Pallet::<T>::bond(RawOrigin::Signed(caller.clone()).into(), signature, address, withdraw);
	}: _(RawOrigin::Signed(caller))
}
