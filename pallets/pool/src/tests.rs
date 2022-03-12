/*
* This unittest should only test logic function e.g. Storage, Computation
* and not related with Currency e.g. Balances, Transaction Payment
*/

use crate::pool::PackService;
use crate::{mock::*, Error};
use frame_support::{assert_err, assert_ok};

#[test]
fn player_join_pool_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		assert_ok!(PalletPool::join(
			Origin::signed(TEST_ACCOUNTS[0].0.clone()),
			PackService::Basic
		));
	});
}

#[test]
fn player_join_pool_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		assert_ok!(PalletPool::join(
			Origin::signed(TEST_ACCOUNTS[0].0.clone()),
			PackService::Basic
		));
		assert_err!(
			(PalletPool::join(Origin::signed(TEST_ACCOUNTS[0].0.clone()), PackService::Basic)),
			<Error<Test>>::PlayerAlreadyJoin
		);
	})
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
			let max_player = MAX_INGAME_PLAYER;
			assert_ok!(PalletPool::set_max_player(Origin::root(), max_player));
			assert_eq!(PalletPool::max_player(), max_player, "max_player after set not correct");
		}

		{
			run_to_block(20);
			let max_player = MAX_NEW_PLAYER;
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
			let max_player = MAX_NEW_PLAYER + 1;
			assert_err!(
				PalletPool::set_max_player(Origin::signed(TEST_ACCOUNTS[0].0.clone()), max_player),
				frame_support::error::BadOrigin
			);
		}

		// incorrect max_player value
		{
			run_to_block(10);
			let max_player = MAX_NEW_PLAYER + 1;
			assert_err!(
				PalletPool::set_max_player(Origin::root(), max_player),
				<Error<Test>>::ExceedMaxNewPlayer
			);
		}
	})
}

#[test]
fn should_restrict_max_player() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let mut count = 0;
		for account in TEST_ACCOUNTS {
			if count == MAX_PLAYER {
				assert_err!(
					PalletPool::join(Origin::signed(account.0.clone()), PackService::Basic),
					<Error<Test>>::ExceedMaxPlayer
				);
			} else {
				assert_ok!(PalletPool::join(Origin::signed(account.0.clone()), PackService::Basic));
				count = count + 1;
			}
		}
	})
}

#[test]
fn should_move_newplayers_to_ingame() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		assert_ok!(PalletPool::join(
			Origin::signed(TEST_ACCOUNTS[0].0.clone()),
			PackService::Basic
		));

		{
			let new_players_before = PalletPool::new_players();
			let ingame_players_before = PalletPool::ingame_players();
			assert_eq!(new_players_before.len(), 1, "new_players_before length not correct");
			assert_eq!(ingame_players_before.len(), 0, "ingame_players_before length not correct");
		}

		run_to_block(100);
		{
			let new_players_after = PalletPool::new_players();
			let ingame_players_after = PalletPool::ingame_players();
			assert_eq!(new_players_after.len(), 0, "new_players_after length not correct");
			assert_eq!(ingame_players_after.len(), 1, "ingame_players_after length not correct");
		}
	})
}

#[test]
fn leave_pool_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		assert_ok!(
			(PalletPool::join(Origin::signed(TEST_ACCOUNTS[0].0.clone()), PackService::Basic))
		);
		run_to_block(10);
		assert_ok!(PalletPool::leave(Origin::signed(TEST_ACCOUNTS[0].0.clone())));
	})
}

#[test]
fn leave_pool_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		assert_ok!(
			(PalletPool::join(Origin::signed(TEST_ACCOUNTS[0].0.clone()), PackService::Basic))
		);
		run_to_block(15);
		assert_ok!(PalletPool::leave(Origin::signed(TEST_ACCOUNTS[0].0.clone())));
		run_to_block(20);
		assert_err!(
			PalletPool::leave(Origin::signed(TEST_ACCOUNTS[0].0.clone())),
			<Error<Test>>::PlayerNotFound
		);
	})
}
