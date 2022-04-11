/*
* This unittest should only test logic function e.g. Storage, Computation
* and not related with Currency e.g. Balances, Transaction Payment
*/
use crate::{mock::*, Error};
use crate::{ PlayerCount, Tickets};
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::currency::{unit, NativeToken::GAKI};
use gafi_primitives::pool::{GafiPool, Level};
use sp_runtime::AccountId32;
use sp_std::str::FromStr;

fn make_deposit(account: &AccountId32, balance: u128) {
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(balance: u128) -> AccountId32 {
	let ALICE: AccountId32 = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
	make_deposit(&ALICE, balance);
	assert_eq!(Balances::free_balance(&ALICE), balance);
	return ALICE;
}

#[test]
fn player_join_pool_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let count_before = PlayerCount::<Test>::get();
		let alice = new_account(1_000_000 * unit(GAKI));
		assert_ok!(PalletPool::join(alice.clone(), Level::Basic));

		let player = Tickets::<Test>::get(alice);
		assert_ne!(player, None);

		let count_after = PlayerCount::<Test>::get();
		assert_eq!(count_before, count_after - 1);
	});
}



#[test]
fn set_max_player_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		{
			run_to_block(1);
			let max_player = 10;
			assert_ok!(PalletPool::set_max_player(Origin::root(), max_player));
			assert_eq!(PalletPool::max_player(), max_player, "max_player after set not correct");
		}

		{
			run_to_block(10);
			let max_player = MAX_PLAYER;
			assert_ok!(PalletPool::set_max_player(Origin::root(), max_player));
			assert_eq!(PalletPool::max_player(), max_player, "max_player after set not correct");
		}

		{
			run_to_block(20);
			let max_player = MAX_PLAYER;
			assert_ok!(PalletPool::set_max_player(Origin::root(), max_player));
			assert_eq!(PalletPool::max_player(), max_player, "max_player after set not correct");
		}
	})
}

#[test]
fn set_max_player_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		// bad origin
		{
			run_to_block(10);
			let max_player = MAX_PLAYER + 1;
			assert_err!(
				PalletPool::set_max_player(Origin::signed(AccountId32::new([0; 32])), max_player),
				frame_support::error::BadOrigin
			);
		}
	})
}

#[test]
fn should_restrict_max_player() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let max_player = 1000u32;
		assert_ok!(PalletPool::set_max_player(Origin::root(), max_player));
		
		let accounts = || -> Vec<AccountId32> {
			let mut account_vec = Vec::new();
			for i in 0 .. max_player {
				let new_account = AccountId32::new([i as u8; 32]);
				make_deposit(&new_account, 1_000_000 * unit(GAKI));
				account_vec.push(new_account);
			}
			account_vec
		};
		
		let mut count = 0;
		for account in accounts() {
			if count == max_player {
				assert_err!(
					PalletPool::join(account, Level::Basic),
					<Error<Test>>::ExceedMaxPlayer
				);
			} else {
				assert_ok!(PalletPool::join(account, Level::Basic));
				count = count + 1;
			}
		}
	})
}


#[test]
fn new_player_leave_pool_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let alice = new_account(1_000_000 * unit(GAKI));
		assert_ok!(PalletPool::join(
			alice.clone(),
			Level::Basic
		));
		run_to_block(2);
		assert_ok!(PalletPool::leave(alice));
	})
}
