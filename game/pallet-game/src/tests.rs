use crate::{
	mock::*,
	types::{GameDetails, GameMintSettings, Item},
	Error, Event, *,
};
use features::id;
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use gafi_support::common::{unit, NativeToken::GAKI};
use pallet_nfts::{
	CollectionRole, CollectionRoles, CollectionSettings, ItemSettings, MintSettings, MintType,
};
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

fn default_collection_config() -> CollectionConfigFor<Test> {
	GameCollectionConfig {
		settings: CollectionSettings::all_enabled(),
		max_supply: None,
		mint_settings: GameMintSettings::default(),
	}
}

fn collection_config(amount: u32, price: u128) -> CollectionConfigFor<Test> {
	GameCollectionConfig {
		settings: CollectionSettings::all_enabled(),
		max_supply: None,
		mint_settings: GameMintSettings {
			amount: Some(amount),
			mint_settings: MintSettings {
				mint_type: MintType::Issuer,
				price: Some(price),
				start_block: None,
				end_block: None,
				default_item_settings: ItemSettings::all_enabled(),
			},
		},
	}
}

fn default_item_config() -> ItemConfig {
	ItemConfig::default()
}

fn create_items() {
	run_to_block(1);
	let owner = new_account([0; 32], 3 * unit(GAKI));

	assert_ok!(PalletGame::create_game(
		RuntimeOrigin::signed(owner.clone()),
		owner.clone()
	));

	assert_ok!(PalletGame::create_game_colletion(
		RuntimeOrigin::signed(owner.clone()),
		0,
		owner.clone(),
		collection_config(50, 0),
	));

	assert_ok!(PalletGame::create_item(
		RuntimeOrigin::signed(owner.clone()),
		0,
		0,
		default_item_config(),
		100
	));
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
			admin.clone()
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
			admin.clone(),
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
			admin.clone(),
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
			admin.clone(),
		));

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(admin.clone()),
			0,
			admin.clone(),
			default_collection_config(),
		));

		assert_eq!(GameCollections::<Test>::get(0)[0], 0);
		assert_eq!(CollectionGame::<Test>::get(0), Some(0));

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(admin.clone()),
			0,
			admin.clone(),
			default_collection_config(),
		));
		assert_eq!(GameCollections::<Test>::get(0)[1], 1);
	})
}

#[test]
fn create_game_collection_should_fails() {
	new_test_ext().execute_with(|| {
		// no permission
		{
			run_to_block(1);
			let before_balance = 3 * unit(GAKI);
			let owner = new_account([0; 32], before_balance);
			let admin = new_account([1; 32], 3 * unit(GAKI));
			let acc = new_account([3; 32], 3 * unit(GAKI));

			assert_ok!(PalletGame::create_game(
				RuntimeOrigin::signed(owner.clone()),
				admin.clone(),
			));

			// random acc should has no permission
			assert_err!(
				PalletGame::create_game_colletion(
					RuntimeOrigin::signed(acc.clone()),
					0,
					acc.clone(),
					default_collection_config(),
				),
				Error::<Test>::NoPermission
			);

			// game owner should has no permission
			assert_err!(
				PalletGame::create_game_colletion(
					RuntimeOrigin::signed(owner.clone()),
					0,
					owner.clone(),
					default_collection_config(),
				),
				Error::<Test>::NoPermission
			);
		}
	})
}

#[test]
fn create_collection_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let owner = new_account([0; 32], before_balance);

		let admin = new_account([1; 32], 3 * unit(GAKI));

		assert_ok!(PalletGame::create_collection(
			RuntimeOrigin::signed(admin.clone()),
			admin.clone(),
			default_collection_config(),
		));
	})
}

#[test]
fn add_game_collection_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let owner = new_account([0; 32], before_balance);

		let admin = new_account([1; 32], 3 * unit(GAKI));

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			admin.clone(),
		));

		assert_ok!(PalletGame::create_collection(
			RuntimeOrigin::signed(owner.clone()),
			admin.clone(),
			default_collection_config(),
		));

		assert_ok!(PalletGame::create_collection(
			RuntimeOrigin::signed(owner.clone()),
			admin.clone(),
			default_collection_config(),
		));

		assert_ok!(PalletGame::add_game_collection(
			RuntimeOrigin::signed(owner.clone()),
			0,
			[0, 1].to_vec()
		));

		assert_eq!(GameCollections::<Test>::get(0), [0, 1].to_vec());
		assert_eq!(CollectionGame::<Test>::get(0).unwrap(), 0);
		assert_eq!(CollectionGame::<Test>::get(1).unwrap(), 0);
	})
}

#[test]
fn create_item_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let owner = new_account([0; 32], before_balance);

		let admin = new_account([1; 32], 3 * unit(GAKI));

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			admin.clone(),
		));

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(admin.clone()),
			0,
			admin.clone(),
			default_collection_config(),
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0,
			default_item_config(),
			1000
		));

		assert_eq!(ItemReserve::<Test>::get(0).to_vec(), [Item::new(0, 1000)]);
	})
}

#[test]
fn add_item_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let owner = new_account([0; 32], before_balance);

		let admin = new_account([1; 32], 3 * unit(GAKI));

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			admin.clone(),
		));

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(admin.clone()),
			0,
			admin.clone(),
			default_collection_config(),
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0,
			default_item_config(),
			1000
		));
		assert_ok!(PalletGame::add_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0,
			1000
		));

		assert_eq!(ItemReserve::<Test>::get(0).to_vec(), [Item::new(0, 2000)]);
	})
}

#[test]
fn withdraw_reserve_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(2);

		let _ = ItemReserve::<Test>::try_mutate(0, |reserve_vec| {
			let _ = reserve_vec.try_push(Item::new(1, 9));
			let _ = reserve_vec.try_push(Item::new(2, 5));
			let _ = reserve_vec.try_push(Item::new(3, 1));
			Ok(())
		})
		.map_err(|_err: Error<Test>| <Error<Test>>::ExceedMaxItem);

		let item = PalletGame::withdraw_reserve(&0, 0);
		assert_eq!(item.unwrap(), 1);

		let item = PalletGame::withdraw_reserve(&0, 9);
		assert_eq!(item.unwrap(), 2);

		let item = PalletGame::withdraw_reserve(&0, 13);
		assert_eq!(item.unwrap(), 3);
	})
}

#[test]
fn mint_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let owner = new_account([0; 32], before_balance);

		let admin = new_account([1; 32], 3 * unit(GAKI));
		let mint_fee: u128 = 3 * unit(GAKI);

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			admin.clone(),
		));

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(admin.clone()),
			0,
			admin.clone(),
			collection_config(10, mint_fee),
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0,
			default_item_config(),
			1
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			1,
			default_item_config(),
			1
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			2,
			default_item_config(),
			1
		));

		let before_balance = 3000 * unit(GAKI);
		let player = new_account([2; 32], before_balance);
		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			3
		));
		assert_eq!(ItemBalances::<Test>::get((0, player.clone(), 0)), 1);
		assert_eq!(ItemBalances::<Test>::get((0, player.clone(), 1)), 1);
		assert_eq!(ItemBalances::<Test>::get((0, player.clone(), 2)), 1);
		assert_eq!(
			Balances::free_balance(player.clone()),
			before_balance - (mint_fee * 3)
		);
	})
}

#[test]
fn mint_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let owner = new_account([0; 32], 3 * unit(GAKI));

		let admin = new_account([1; 32], 3 * unit(GAKI));
		let mint_fee: u128 = 3 * unit(GAKI);

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			admin.clone(),
		));

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(admin.clone()),
			0,
			admin.clone(),
			collection_config(9, mint_fee),
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0,
			default_item_config(),
			10
		));

		let player = new_account([2; 32], 3000 * unit(GAKI));
		assert_err!(
			PalletGame::mint(RuntimeOrigin::signed(player.clone()), 0, player.clone(), 10),
			Error::<Test>::ExceedAllowedAmount
		);

		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			9
		));

		// one left
		assert_err!(
			PalletGame::mint(RuntimeOrigin::signed(player.clone()), 0, player.clone(), 4),
			Error::<Test>::ExceedTotalAmount
		);

		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			1
		));

		assert_err!(
			PalletGame::mint(RuntimeOrigin::signed(player.clone()), 0, player.clone(), 1),
			Error::<Test>::SoldOut
		);
	})
}

#[test]
pub fn burn_items_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let owner = new_account([0; 32], 3 * unit(GAKI));

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			owner.clone()
		));

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(owner.clone()),
			0,
			owner.clone(),
			collection_config(50, 0),
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(owner.clone()),
			0,
			0,
			default_item_config(),
			100
		));

		let player = new_account([2; 32], 3000 * unit(GAKI));
		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			10
		));
		assert_eq!(ItemBalances::<Test>::get((0, player.clone(), 0)), 10);

		assert_ok!(PalletGame::burn(
			RuntimeOrigin::signed(player.clone()),
			0,
			0,
			5
		));
		assert_eq!(ItemBalances::<Test>::get((0, player.clone(), 0)), 5);

		assert_err!(
			PalletGame::burn(RuntimeOrigin::signed(player.clone()), 0, 0, 6),
			Error::<Test>::InsufficientItemBalance
		);
	})
}

#[test]
pub fn transfer_item_should_works() {
	new_test_ext().execute_with(|| {
		create_items();

		let player = new_account([2; 32], 3000 * unit(GAKI));
		let dest = new_account([3; 32], 3000 * unit(GAKI));
		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			10
		));

		assert_ok!(PalletGame::transfer(
			RuntimeOrigin::signed(player.clone()),
			0,
			0,
			dest.clone(),
			5
		));

		assert_eq!(ItemBalances::<Test>::get((0, player.clone(), 0)), 5);
		assert_eq!(ItemBalances::<Test>::get((0, dest.clone(), 0)), 5);

		assert_err!(
			PalletGame::transfer(RuntimeOrigin::signed(player.clone()), 0, 0, dest.clone(), 6),
			Error::<Test>::InsufficientItemBalance
		);
	})
}

#[test]
pub fn set_upgrade_item_should_works() {
	new_test_ext().execute_with(|| {
		

	})
}
