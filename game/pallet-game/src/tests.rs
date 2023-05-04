use crate::{mock::*, types::GameDetails, Error, Event, *};
use features::id;
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use gafi_support::{
	common::{unit, NativeToken::GAKI},
	game::Support,
};
use pallet_nfts::{CollectionRole, CollectionRoles, CollectionSettings, MintSettings};
use sp_runtime::AccountId32;

fn make_deposit(account: &AccountId32, balance: u128) {
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(account: [u8; 32], balance: u128) -> AccountId32 {
	let acc: AccountId32 = AccountId32::from(account);
	make_deposit(&acc, balance);
	assert_eq!(Balances::free_balance(&acc), balance);
	return acc
}

#[test]
fn create_first_game_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let owner = new_account([0; 32], before_balance);

		let admin = new_account([1; 32], 3 * unit(GAKI));

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			Some(admin.clone())
		));

		let game = Games::<Test>::get(0).unwrap();
		assert_eq!(game.owner, owner);
		assert_eq!(game.collections, 0);
		assert_eq!(game.owner_deposit, GAME_DEPOSIT_VAL);
		assert_eq!(NextGameId::<Test>::get(), Some(1));
		assert_eq!(
			Balances::free_balance(owner),
			before_balance - GAME_DEPOSIT_VAL
		);
		assert_eq!(
			GameRoleOf::<Test>::get(0, admin).unwrap(),
			CollectionRoles(
				CollectionRole::Issuer | CollectionRole::Freezer | CollectionRole::Admin
			)
		);
	});
}

#[test]
fn set_swap_fee_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let fee = Percent::from_parts(30);
		let start_block = 100;

		let owner = new_account([0; 32], before_balance);
		let admin = new_account([1; 32], 3 * unit(GAKI));
		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			Some(admin.clone())
		));

		assert_ok!(PalletGame::set_swap_fee(
			RuntimeOrigin::signed(admin),
			0,
			fee,
			start_block
		));
		assert_eq!(SwapFee::<Test>::get(0).unwrap(), (fee, start_block));
	})
}

#[test]
fn set_swap_fee_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let fee = Percent::from_parts(30);
		let start_block = 100;

		let owner = new_account([0; 32], before_balance);
		let admin = new_account([1; 32], 3 * unit(GAKI));
		let not_admin = new_account([2; 32], 3 * unit(GAKI));

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			Some(admin.clone())
		));

		assert_err!(
			PalletGame::set_swap_fee(RuntimeOrigin::signed(not_admin), 0, fee, start_block),
			<Error<Test>>::NoPermission
		);

		let invalid_fee = Percent::from_parts(31);
		assert_err!(
			PalletGame::set_swap_fee(RuntimeOrigin::signed(admin), 0, invalid_fee, start_block),
			<Error<Test>>::SwapFeeTooHigh
		);
	})
}

#[test]
fn create_game_collection_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let owner = new_account([0; 32], before_balance);

		let admin = new_account([1; 32], 3 * unit(GAKI));

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			Some(admin.clone())
		));

		let config = CollectionConfig {
			settings: CollectionSettings::all_enabled(),
			max_supply: None,
			mint_settings: MintSettings::default(),
		};

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(admin.clone()),
			0,
			Some(admin.clone()),
			config
		));

		assert_eq!(GameCollections::<Test>::get(0).unwrap()[0], 0);

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(admin.clone()),
			0,
			Some(admin),
			config
		));
		assert_eq!(GameCollections::<Test>::get(0).unwrap()[1], 1);
	})
}
