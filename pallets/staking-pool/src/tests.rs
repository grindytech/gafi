/*
* This unittest should only test logic function e.g. Storage, Computation
* and not related with Currency e.g. Balances, Transaction Payment
*/
use crate::{mock::*};
use crate::{PlayerCount, Tickets};
use frame_support::{assert_ok, traits::Currency};
use gafi_primitives::currency::{unit, NativeToken::GAKI};
use gafi_primitives::{ticket::TicketLevel, system_services::SystemPool, constant::ID};
use sp_runtime::AccountId32;
use sp_std::str::FromStr;

const STAKING_BASIC_ID: ID = [223, 236, 215, 227, 124, 27, 202, 81, 144, 36, 86, 22, 116, 218, 112, 227, 22, 53, 161, 192, 104, 124, 153, 71, 95, 117, 111, 122, 147, 230, 110, 79];
const STAKING_MEDIUM_ID: ID = [177, 56, 207, 179, 85, 59, 155, 11, 173, 173, 83, 172, 196, 240, 181, 27, 237, 209, 38, 28, 156, 42, 201, 201, 44, 55, 39, 85, 5, 86, 245, 170];
const STAKING_ADVANCE_ID: ID = [101, 218, 57, 134, 234, 236, 240, 70, 203, 44, 65, 103, 58, 237, 157, 78, 30, 102, 23, 48, 220, 49, 198, 47, 50, 125, 245, 209, 89, 51, 89, 93];

fn make_deposit(account: &AccountId32, balance: u128) {
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(balance: u128) -> AccountId32 {
	let alice: AccountId32 =
		AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
	make_deposit(&alice, balance);
	assert_eq!(Balances::free_balance(&alice), balance);
	return alice;
}

fn _new_accounts(count: u32, balance: u128) -> Vec<AccountId32> {
	let mut account_vec = Vec::new();
	for i in 0..count {
		let new_account = AccountId32::new([i as u8; 32]);
		make_deposit(&new_account, balance);
		account_vec.push(new_account);
	}
	account_vec
}

#[test]
fn default_services_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		assert_eq!(StakingPool::get_service(STAKING_BASIC_ID).is_none(), false);
		assert_eq!(StakingPool::get_service(STAKING_MEDIUM_ID).is_none(), false);
		assert_eq!(StakingPool::get_service(STAKING_ADVANCE_ID).is_none(), false);
	})
}

#[test]
fn player_join_pool_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let count_before = PlayerCount::<Test>::get();
		let alice = new_account(1_000_000 * unit(GAKI));
		assert_ok!(StakingPool::join(alice.clone(), STAKING_BASIC_ID));

		let player = Tickets::<Test>::get(alice);
		assert_ne!(player, None);

		let count_after = PlayerCount::<Test>::get();
		assert_eq!(count_before, count_after - 1);
	});
}

#[test]
fn leave_pool_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let alice = new_account(1_000_000 * unit(GAKI));
		assert_ok!(StakingPool::join(alice.clone(), STAKING_BASIC_ID));
		run_to_block(2);
		assert_ok!(StakingPool::leave(alice, STAKING_BASIC_ID));
	})
}
