use crate::{mock::*, Pallet};
use frame_support::traits::Currency;
use gafi_primitives::{
	common::{constant::ID,
	currency::{unit, NativeToken::GAKI}},
};
use sp_core::H160;
use sp_runtime::{AccountId32, Permill};
use sp_std::str::FromStr;

fn make_deposit(account: &AccountId32, balance: u128) {
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn _new_account(account: [u8; 32], balance: u128) -> AccountId32 {
	let acc: AccountId32 = AccountId32::from(account);
	make_deposit(&acc, balance);
	assert_eq!(Balances::free_balance(&acc), balance);
	return acc
}

#[test]
fn correct_and_deposit_fee_funding_works() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id: ID = [0_u8; 32];
		let pool = AccountId32::from(pool_id);
		let pool_balance = 100 * unit(GAKI);
		let service_fee = 10 * unit(GAKI);
		make_deposit(&pool, pool_balance);

		let targets = vec![H160::from_str("0x0A6617b82B594C83240092BDc86E2e16354d1456").unwrap()];
		let target: H160 = H160::from_str("0x0A6617b82B594C83240092BDc86E2e16354d1456").unwrap();
		let discount = Permill::from_percent(40);

		let funding_fee = Pallet::<Test>::correct_and_deposit_fee_funding(
			pool_id,
			targets,
			target,
			service_fee,
			discount,
		)
		.unwrap();

		assert_eq!(funding_fee, 6 * unit(GAKI));
		assert_eq!(Balances::free_balance(&pool), 96 * unit(GAKI));
	})
}

#[test]
fn correct_and_deposit_fee_funding_should_return_none() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id: ID = [0_u8; 32];
		let pool = AccountId32::from(pool_id);
		let pool_balance = 100 * unit(GAKI);
		let service_fee = 10 * unit(GAKI);
		make_deposit(&pool, pool_balance);
		let discount = Permill::from_percent(40);

		let targets = vec![
			H160::from_str("0x0A6617b82B594C83240092BDc86E2e16354d1456").unwrap(),
			H160::from_str("0x0A6617b82B594C83240092BDc86E2e16354d1247").unwrap(),
		];
		let target: H160 = H160::from_str("0x0A6617b82B594C83240092BDc86E2e16354d8482").unwrap();

		let funding_fee = Pallet::<Test>::correct_and_deposit_fee_funding(
			pool_id,
			targets,
			target,
			service_fee,
			discount,
		);

		assert!(funding_fee.is_none());
		assert_eq!(Balances::free_balance(&pool), pool_balance);
	})
}
