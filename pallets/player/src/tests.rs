use crate::{mock::*, Config, Error, PlayerOwned, Players};
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	players::{PlayerJoinedPoolStatistic, PlayersTime},
	system_services::SystemPool,
};

const START_BLOCK: u64 = 10;

#[test]
fn gen_id_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(START_BLOCK);
		let id = PalletGame::gen_id().unwrap();
		assert_eq!(id.len(), 32, "id not correct");
	});
}

#[test]
fn create_new_player_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(START_BLOCK);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1000_000);
		let user_name = [0u8; 16];
		assert_ok!(PalletGame::create_new_player(ALICE, user_name));
	});
}

#[test]
fn create_new_player_should_fail() {
	new_test_ext().execute_with(|| {
		run_to_block(START_BLOCK);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1000_000);
		let user_name = [0u8; 16];
		assert_ok!(PalletGame::create_new_player(ALICE, user_name));
		assert_err!(
			PalletGame::create_new_player(ALICE, user_name),
			<Error<Test>>::PlayerExisted
		);
	});
}

#[test]
fn is_player_id_check_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(START_BLOCK);
		let id = PalletGame::gen_id().unwrap();
		let check = Players::<Test>::get(&id).is_none();
		assert_eq!(check, true, "player id should available");
		let user_name = [0u8; 16];
		let player_id = PalletGame::create_new_player(ALICE, user_name).unwrap();
		let check = Players::<Test>::get(&player_id).is_none();
		assert_eq!(check, false, "player id should not available");
	});
}

#[test]
fn is_player_check_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(START_BLOCK);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1000_000);

		let check = PlayerOwned::<Test>::get(&ALICE).is_none();
		assert_eq!(check, true, "player should available");

		let user_name = [1u8; 16];
		assert_ok!(PalletGame::create_player(Origin::signed(ALICE), user_name));

		let check = PlayerOwned::<Test>::get(&ALICE).is_none();
		assert_eq!(check, false, "player should not available");

		assert_err!(
			PalletGame::create_player(Origin::signed(ALICE), user_name),
			<Error<Test>>::PlayerExisted
		);

		run_to_block(10);
	});
}

#[test]
fn get_total_time_joined_upfront_should_return_zero() {
	new_test_ext().execute_with(|| {
		run_to_block(START_BLOCK);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1_000_000 * unit(GAKI));

		assert_eq!(PalletGame::get_total_time_joined_upfront(&ALICE), 0);
	});
}

#[test]
fn get_total_time_joined_upfront_should_work() {
	new_test_ext().execute_with(|| {
		run_to_block(START_BLOCK);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1_000_000 * unit(GAKI));
		let _result = <Test as Config>::UpfrontPool::join(ALICE, UPFRONT_BASIC_ID);

		assert_eq!(PalletGame::get_total_time_joined_upfront(&ALICE), 0);

		run_to_block(START_BLOCK + 10);

		assert_eq!(
			PalletGame::get_total_time_joined_upfront(&ALICE),
			(MILLISECS_PER_BLOCK * 10).into()
		);
	});
}

#[test]
fn add_time_joined_upfront_should_work() {
	new_test_ext().execute_with(|| {
		run_to_block(START_BLOCK);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1_000_000 * unit(GAKI));
		let _result = GafiMembership::registration(Origin::signed(ALICE));

		PalletGame::add_time_joined_upfront(ALICE, 100);

		assert_eq!(PalletGame::total_time_joined_upfront(ALICE).unwrap(), 100);
	});
}

#[test]
fn add_time_joined_upfront_should_add_with_existed_player_time() {
	new_test_ext().execute_with(|| {
		run_to_block(START_BLOCK);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1_000_000 * unit(GAKI));
		let _result = GafiMembership::registration(Origin::signed(ALICE));
		let _result = <Test as Config>::UpfrontPool::join(ALICE, UPFRONT_BASIC_ID);

		run_to_block(START_BLOCK + 10);
		let _result = <Test as Config>::UpfrontPool::leave(ALICE);

		PalletGame::add_time_joined_upfront(ALICE, 100);

		assert_eq!(
			PalletGame::total_time_joined_upfront(ALICE).unwrap(),
			100u128.saturating_add((MILLISECS_PER_BLOCK * 10).into())
		);
	});
}
