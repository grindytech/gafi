use crate::{mock::*, types::GameDetails, Error, Event, *};
use features::id;
use frame_support::{assert_noop, assert_ok, traits::Currency};
use gafi_support::{
	common::{unit, NativeToken::GAKI},
	game::Support,
};
use pallet_nfts::{CollectionRole, CollectionRoles};
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
