use crate::mock::*;
use frame_support::{assert_ok, traits::Currency};
use gafi_primitives::system_services::SystemPool;
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	ticket::{TicketLevel, TicketType, SystemTicket},
};
use gafi_tx::Config;
use rand::prelude::*;
use sp_runtime::AccountId32;
use sp_std::str::FromStr;

const CIRCLE_BLOCK: u64 = (TIME_SERVICE as u64) / SLOT_DURATION;
const ADDITIONAL_BLOCK: u64 = 1;
const LEVELS: [TicketLevel; 3] = [TicketLevel::Basic, TicketLevel::Medium, TicketLevel::Advance];

fn init_join_pool(pool_fee: u128, ticket: TicketType) {
	let sender = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap(); //ALICE

	let base_balance = 1_000_000 * unit(GAKI);
	let _ = <Test as Config>::Currency::deposit_creating(&sender, base_balance);
	{
		assert_eq!(
			<Test as Config>::Currency::free_balance(sender.clone()),
			base_balance
		);
	}

	let before_balance = <Test as Config>::Currency::free_balance(sender.clone());
	assert_ok!(Pool::join(Origin::signed(sender.clone()), ticket));
	assert_eq!(
		<Test as Config>::Currency::free_balance(sender.clone()),
		before_balance - (pool_fee * 2)
	); //charge x2 when once join

	for i in 1..10 {
		run_to_block((CIRCLE_BLOCK * i) + ADDITIONAL_BLOCK);
		assert_eq!(
			<Test as Config>::Currency::free_balance(sender.clone()),
			before_balance - (pool_fee * (i + 1) as u128)
		);
	}
}

#[test]
fn charge_join_pool_basic_work() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_fee = UpfrontPool::get_service(TicketLevel::Basic).unwrap();
		init_join_pool(pool_fee.value, TicketType::System(SystemTicket::Upfront(TicketLevel::Basic)) );
	})
}

#[test]
fn charge_join_pool_medium_work() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_fee = UpfrontPool::get_service(TicketLevel::Medium).unwrap();
		init_join_pool(pool_fee.value, TicketType::System(SystemTicket::Upfront(TicketLevel::Medium)) );
	})
}

#[test]
fn charge_join_max_pool_work() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_fee = UpfrontPool::get_service(TicketLevel::Advance).unwrap();
		init_join_pool(pool_fee.value, TicketType::System(SystemTicket::Upfront(TicketLevel::Advance)) );
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
		assert_ok!(Pool::leave(Origin::signed(sender.clone()), None));

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
fn leave_pool_early_works() {
	for i in 0..10 {
		for level in LEVELS {
			ExtBuilder::default().build_and_execute(|| {
				let pool_fee = UpfrontPool::get_service(level).unwrap();
				let mut rng = thread_rng();
				let leave_block = rng.gen_range(2..CIRCLE_BLOCK);
				init_leave_pool(
					i,
					pool_fee.value,
					TicketType::System(SystemTicket::Upfront(level)),
					1,
					leave_block,
				);
			});
		}
	}
}

#[test]
fn leave_pool_over_works() {
	for i in 0..10 {
		for level in LEVELS {
			ExtBuilder::default().build_and_execute(|| {
				let pool_fee = UpfrontPool::get_service(level).unwrap();
				let mut rng = thread_rng();
				let start_block = rng.gen_range(1..CIRCLE_BLOCK);
				let leave_block = rng.gen_range((CIRCLE_BLOCK * 2)..(CIRCLE_BLOCK * 4));
				init_leave_pool(
					i,
					pool_fee.value,
					TicketType::System(SystemTicket::Upfront(level)),
					start_block,
					leave_block,
				);
			});
		}
	}
}
