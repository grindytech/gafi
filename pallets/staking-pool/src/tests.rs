/*
* This unittest should only test logic function e.g. Storage, Computation
* and not related with Currency e.g. Balances, Transaction Payment
*/
use crate::{mock::*};
use crate::{PlayerCount, Tickets};
use codec::Encode;
use frame_support::{assert_ok, traits::Currency};
use gafi_primitives::currency::{unit, NativeToken::GAKI};
use gafi_primitives::{system_services::SystemPool, constant::ID};
use sp_core::blake2_256;
use sp_runtime::AccountId32;
use sp_std::str::FromStr;

const STAKING_BASIC_ID: ID = [0_u8; 32];
const STAKING_MEDIUM_ID: ID = [1_u8; 32];
const STAKING_ADVANCE_ID: ID = [2_u8; 32];

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
		assert_eq!(StakingPool::get_service(STAKING_BASIC_ID.using_encoded(blake2_256)).is_none(), false);
		assert_eq!(StakingPool::get_service(STAKING_MEDIUM_ID.using_encoded(blake2_256)).is_none(), false);
		assert_eq!(StakingPool::get_service(STAKING_ADVANCE_ID.using_encoded(blake2_256)).is_none(), false);
	})
}

#[test]
fn player_join_pool_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let count_before = PlayerCount::<Test>::get();
		let alice = new_account(1_000_000 * unit(GAKI));
		assert_ok!(StakingPool::join(alice.clone(), STAKING_BASIC_ID.using_encoded(blake2_256)));

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
		assert_ok!(StakingPool::join(alice.clone(), STAKING_BASIC_ID.using_encoded(blake2_256)));
		run_to_block(2);
		assert_ok!(StakingPool::leave(alice.clone()));
		assert_eq!(Tickets::<Test>::get(alice.clone()), None);
	})
}
