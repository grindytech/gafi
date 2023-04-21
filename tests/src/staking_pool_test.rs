use crate::mock::*;
use frame_support::{assert_ok, traits::Currency};
use gafi_support::{
	common::constant::ID,
	pool::system_services::SystemPool,
};
use gafi_tx::Config;
use sp_runtime::AccountId32;
use gu_mock::one_mil_gaki;

const TICKETS: [ID; 3] = [STAKING_BASIC_ID, STAKING_MEDIUM_ID, STAKING_ADVANCE_ID];

fn join_pool(account: AccountId32, pool_id: ID) {
	let base_balance = one_mil_gaki();

	let staking_amount = StakingPool::get_service(pool_id).unwrap().value;
	let _ = <Test as Config>::Currency::deposit_creating(&account, base_balance);

	{
		assert_eq!(
			<Test as Config>::Currency::free_balance(account.clone()),
			base_balance
		);
	}

	assert_ok!(Pool::join(RuntimeOrigin::signed(account.clone()), pool_id));
	assert_eq!(
		<Test as Config>::Currency::free_balance(account.clone()),
		base_balance - staking_amount
	);
}

fn leave_pool(account: AccountId32, pool_id: ID) {
	let before_balance = <Test as Config>::Currency::free_balance(account.clone());
	let staking_amount = StakingPool::get_service(pool_id).unwrap().value;

	assert_ok!(Pool::leave(RuntimeOrigin::signed(account.clone()), pool_id));
	assert_eq!(
		<Test as Config>::Currency::free_balance(account.clone()),
		before_balance + staking_amount
	);
}

#[test]
fn join_pool_works() {
	for i in 0..TICKETS.len() {
		ExtBuilder::default().build_and_execute(|| {
			let account = AccountId32::new([i as u8; 32]);

			join_pool(account, TICKETS[i]);
		})
	}
}

#[test]
fn leave_pool_works() {
	for i in 0..TICKETS.len() {
		ExtBuilder::default().build_and_execute(|| {
			let account = AccountId32::new([i as u8; 32]);
			let ticket = TICKETS[i];

			join_pool(account.clone(), ticket);
			leave_pool(account.clone(), ticket);
		})
	}
}
