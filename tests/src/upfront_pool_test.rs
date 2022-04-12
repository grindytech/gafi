use crate::mock::*;
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	pool::{GafiPool, Level, TicketType},
};
use gafi_tx::Config;
use hex_literal::hex;
use rand::prelude::*;
use sp_core::H160;
use sp_runtime::AccountId32;
use sp_std::str::FromStr;

const CIRCLE_BLOCK: u64 = (TIME_SERVICE as u64) / SLOT_DURATION;
const ADDITIONAL_BLOCK: u64 = 1;
const LEVELS: [Level; 3] = [Level::Basic, Level::Medium, Level::Max];

fn init_join_pool(pool_fee: u128, ticket: TicketType, is_bond: bool) {
	let sender = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap(); //ALICE

	let base_balance = 1_000_000 * unit(GAKI);
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
	assert_ok!(Pool::join(Origin::signed(sender.clone()), ticket));
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
		let pool_fee = StakingPool::get_service(Level::Basic);
		init_join_pool(pool_fee.value, TicketType::Staking(Level::Basic), false);
	})
}

#[test]
fn charge_join_pool_medium_work() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_fee = StakingPool::get_service(Level::Medium);
		init_join_pool(pool_fee.value, TicketType::Staking(Level::Medium), false);
	})
}

#[test]
fn charge_join_max_pool_work() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_fee = StakingPool::get_service(Level::Max);
		init_join_pool(pool_fee.value, TicketType::Staking(Level::Max), false);
	})
}

fn init_leave_pool(
	index: i32,
	pool_fee: u128,
	ticket: TicketType,
	start_block: u64,
	leave_block: u64,
) {
	let sender = AccountId32::new([index as u8; 32]);

	let base_balance = 1_000_000 * unit(GAKI);
	let _ = <Test as Config>::Currency::deposit_creating(&sender, base_balance);
	let original_balance = <Test as Config>::Currency::free_balance(sender.clone());

	run_to_block(start_block);
	assert_ok!(Pool::join(Origin::signed(sender.clone()), ticket));
	assert_eq!(
		<Test as Config>::Currency::free_balance(sender.clone()),
		original_balance - (pool_fee * 2)
	); //charge x2 when once join

	run_to_block(leave_block);
	{
		let before_balance = <Test as Config>::Currency::free_balance(sender.clone());
		assert_ok!(Pool::leave(Origin::signed(sender.clone())));

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
	for i in 0..50 {
		ExtBuilder::default().build_and_execute(|| {
			for level in LEVELS {
				let pool_fee = StakingPool::get_service(level);
				let mut rng = thread_rng();
				let leave_block = rng.gen_range(2..CIRCLE_BLOCK);
				init_leave_pool(i, pool_fee.value, TicketType::Staking(level), 1, leave_block);
			}
		});
	}
}

#[test]
fn leave_basic_pool_over_works() {
	for i in 0..50 {
		ExtBuilder::default().build_and_execute(|| {
			for level in LEVELS {
				let pool_fee = StakingPool::get_service(level);
				let mut rng = thread_rng();
				let start_block = rng.gen_range(1..CIRCLE_BLOCK);
				let leave_block = rng.gen_range((CIRCLE_BLOCK * 2)..(CIRCLE_BLOCK * 4));
				init_leave_pool(
					i,
					pool_fee.value,
					TicketType::Staking(level),
					start_block,
					leave_block,
				);
			}
		});
	}
}
