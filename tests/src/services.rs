use crate::mock::*;
use gafi_primitives::{currency::{NativeToken::AUX, unit}};
use frame_support::{assert_err, assert_ok, traits::Currency};
use hex_literal::hex;
use gafi_primitives::option_pool::PackService;
use pallet_tx_handler::Config;
use rand::prelude::*;
use sp_core::H160;
use sp_runtime::AccountId32;
use sp_std::str::FromStr;

const CIRCLE_BLOCK: u64 = (TIME_SERVICE as u64) / SLOT_DURATION;
const ADDITIONAL_BLOCK: u64 = 1;
static UNIT: u128 = 1_000_000_000_000_000_000;
static CENTI: u128 = UNIT / 100;
static BASIC: u128 = 75 * CENTI;
static MEDIUM: u128 = 75 * CENTI * 2;
static MAX: u128 = 75 * CENTI * 3;

fn init_join_pool(pool_fee: u128, pack: PackService, is_bond: bool) {
	let sender = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap(); //ALICE

	let base_balance = 1_000_000 * unit(AUX);
	let _ = <Test as Config>::Currency::deposit_creating(&sender, base_balance);
	{
		assert_eq!(<Test as Config>::Currency::free_balance(sender.clone()), base_balance);
	}

	if is_bond {
		let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		assert_ok!(PalletAddressMapping::bond(
			Origin::signed(sender.clone()),
			signature,
			address,
			false
		));
	}
	assert_ok!(PalletPool::join(Origin::signed(sender.clone()), pack));
	assert_eq!(
		<Test as Config>::Currency::free_balance(sender.clone()),
		base_balance - (pool_fee * 2)
	); //charge x2 when once join

	for i in 1..10 {
		run_to_block((CIRCLE_BLOCK * i) + ADDITIONAL_BLOCK);
		// let _now = pallet_timestamp::Pallet::<Test>::get();
		assert_eq!(
			<Test as Config>::Currency::free_balance(sender.clone()),
			base_balance - (pool_fee * (i + 1) as u128)
		);
	}
}

#[test]
fn charge_join_pool_basic_work() {
	ExtBuilder::default().build_and_execute(|| {
		init_join_pool(BASIC, PackService::Basic, false);
	})
}

#[test]
fn charge_join_pool_medium_work() {
	ExtBuilder::default().build_and_execute(|| {
		init_join_pool(MEDIUM, PackService::Medium, true);
	})
}

#[test]
fn charge_join_max_pool_work() {
	ExtBuilder::default().build_and_execute(|| {
		init_join_pool(MAX, PackService::Max, true);
	})
}

fn init_leave_pool(
	pool_fee: u128,
	pack: PackService,
	is_bond: bool,
	start_block: u64,
	leave_block: u64,
) {
	let sender = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap(); //ALICE

	let base_balance = 1_000_000 * unit(AUX);
	let _ = <Test as Config>::Currency::deposit_creating(&sender, base_balance);
	{
		assert_eq!(<Test as Config>::Currency::free_balance(sender.clone()), base_balance);
	}

	if is_bond {
		let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		assert_ok!(PalletAddressMapping::bond(
			Origin::signed(sender.clone()),
			signature,
			address,
			false
		));
	}
	run_to_block(start_block);
	assert_ok!(PalletPool::join(Origin::signed(sender.clone()), pack));
	assert_eq!(
		<Test as Config>::Currency::free_balance(sender.clone()),
		base_balance - (pool_fee * 2)
	); //charge x2 when once join

	run_to_block(leave_block);
	{
		let before_balance = <Test as Config>::Currency::free_balance(sender.clone());
		assert_ok!(PalletPool::leave(Origin::signed(sender.clone())));

		let after_balance = <Test as Config>::Currency::free_balance(sender.clone());

		let refund_balance = after_balance - before_balance;

		let block_range = leave_block - start_block;

		if block_range < CIRCLE_BLOCK {
			assert_eq!(refund_balance, pool_fee);
		} else {
			let block_remain = CIRCLE_BLOCK - block_range % CIRCLE_BLOCK;
			let block_rate = (block_remain as u128).saturating_mul(pool_fee);
			let fee_rate = (CIRCLE_BLOCK as u128).saturating_mul(refund_balance);
			assert_eq!(block_rate, fee_rate);
		}
	}
}

#[test]
fn leave_basic_pool_early_works() {
	for _ in 0..50 {
		ExtBuilder::default().build_and_execute(|| {
			let mut rng = thread_rng();
			let leave_block = rng.gen_range(2..CIRCLE_BLOCK);
			init_leave_pool(BASIC, PackService::Basic, true, 1, leave_block);
		});
	}
}

#[test]
fn leave_basic_pool_over_works() {
	for _ in 0..50 {
		ExtBuilder::default().build_and_execute(|| {
			let mut rng = thread_rng();
			let start_block = rng.gen_range(1..CIRCLE_BLOCK);
			let leave_block = rng.gen_range((CIRCLE_BLOCK * 2)..(CIRCLE_BLOCK * 4));

			init_leave_pool(BASIC, PackService::Basic, true, start_block, leave_block);
		});
	}
}
