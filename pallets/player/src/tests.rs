use crate::{mock::*, Config, Error};
use frame_support::{assert_err, assert_ok, traits::Currency};

#[test]
fn gen_id_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let id = PalletGame::gen_id().unwrap();
		assert_eq!(id.len(), 32, "id not correct");
	});
}

#[test]
fn create_new_player_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1000_000);
		let user_name = [0u8; 16];
		assert_ok!(PalletGame::create_new_player(ALICE, user_name));
	});
}

#[test]
fn create_new_player_should_fail() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1000_000);
		let user_name = [0u8; 16];
		assert_ok!(PalletGame::create_new_player(ALICE, user_name));
		assert_err!(PalletGame::create_new_player(ALICE, user_name), <Error<Test>>::PlayerExisted);
	});
}

#[test]
fn is_player_id_check_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let id = PalletGame::gen_id().unwrap();
		let check = PalletGame::is_player_id_available(&id);
		assert_eq!(check, true, "player id should available");
		let user_name = [0u8; 16];
		let player_id = PalletGame::create_new_player(ALICE, user_name).unwrap();
		let check = PalletGame::is_player_id_available(&player_id);
		assert_eq!(check, false, "player id should not available");
	});
}

#[test]
fn is_player_check_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1000_000);

		let check = PalletGame::is_player_available(&ALICE);
		assert_eq!(check, true, "player should available");

		let user_name = [1u8; 16];
		assert_ok!(PalletGame::create_player(Origin::signed(ALICE), user_name));

		let check = PalletGame::is_player_available(&ALICE);
		assert_eq!(check, false, "player should not available");

		assert_err!(
			PalletGame::create_player(Origin::signed(ALICE), user_name),
			<Error<Test>>::PlayerExisted
		);

		run_to_block(10);
	});
}
