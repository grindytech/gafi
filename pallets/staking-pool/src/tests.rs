/*
* This unittest should only test logic function e.g. Storage, Computation
* and not related with Currency e.g. Balances, Transaction Payment
*/
use crate::{mock::*, Error};
use crate::{PlayerCount, Tickets};
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::currency::{unit, NativeToken::GAKI};
use gafi_primitives::{ticket::TicketLevel, system_services::SystemPool};
use sp_runtime::AccountId32;
use sp_std::str::FromStr;

fn make_deposit(account: &AccountId32, balance: u128) {
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(balance: u128) -> AccountId32 {
	let ALICE: AccountId32 =
		AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
	make_deposit(&ALICE, balance);
	assert_eq!(Balances::free_balance(&ALICE), balance);
	return ALICE;
}

fn new_accounts(count: u32, balance: u128) -> Vec<AccountId32> {
	let mut account_vec = Vec::new();
	for i in 0..count {
		let new_account = AccountId32::new([i as u8; 32]);
		make_deposit(&new_account, balance);
		account_vec.push(new_account);
	}
	account_vec
}

#[test]
fn player_join_pool_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let count_before = PlayerCount::<Test>::get();
		let alice = new_account(1_000_000 * unit(GAKI));
		assert_ok!(StakingPool::join(alice.clone(), TicketLevel::Basic));

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
		assert_ok!(StakingPool::join(alice.clone(), TicketLevel::Basic));
		run_to_block(2);
		assert_ok!(StakingPool::leave(alice));
	})
}
