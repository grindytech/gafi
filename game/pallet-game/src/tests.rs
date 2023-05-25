use crate::{
	mock::*,
	types::{Item, *},
	Error, *,
};
use sp_core::sr25519;
use sp_keystore::{testing::KeyStore, SyncCryptoStore};

use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_support::{
	common::{unit, NativeToken::GAKI},
	game::Package,
};
use pallet_nfts::{
	CollectionRole, CollectionRoles, CollectionSettings, ItemSettings, MintSettings, MintType,
};

fn make_deposit(account: &sr25519::Public, balance: u128) {
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(account: u32, balance: u128) -> sr25519::Public {
	let keystore = KeyStore::new();
	let acc: sr25519::Public = keystore
		.sr25519_generate_new(sp_runtime::KeyTypeId::from(account), None)
		.unwrap();
	make_deposit(&acc, balance);
	return acc
}

fn default_collection_config() -> CollectionConfigFor<Test> {
	CollectionConfig {
		settings: CollectionSettings::all_enabled(),
		max_supply: None,
		mint_settings: MintSettings::default(),
	}
}

fn collection_config(price: u128) -> CollectionConfigFor<Test> {
	CollectionConfig {
		settings: CollectionSettings::all_enabled(),
		max_supply: None,
		mint_settings: MintSettings {
			mint_type: MintType::Issuer,
			price: Some(price),
			start_block: None,
			end_block: None,
			default_item_settings: ItemSettings::all_enabled(),
		},
	}
}

fn default_item_config() -> ItemConfig {
	ItemConfig::default()
}

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

fn do_create_game() -> (sr25519::Public, sr25519::Public) {
	let owner: sr25519::Public = new_account(0, 10_000 * unit(GAKI));
	let admin = new_account(1, 10_000 * unit(GAKI));
	assert_ok!(PalletGame::create_game(
		RuntimeOrigin::signed(owner.clone()),
		admin.clone()
	));
	(owner, admin)
}

fn do_create_collection(
	game: u32,
	collection_config: &CollectionConfigFor<Test>,
	admin: &sr25519::Public,
) {
	assert_ok!(PalletGame::create_game_collection(
		RuntimeOrigin::signed(admin.clone()),
		game,
		*collection_config,
	));
}

fn do_create_item(
	admin: &sr25519::Public,
	collection: u32,
	item: u32,
	item_config: &ItemConfig,
	amount: u32,
) {
	assert_ok!(PalletGame::create_item(
		RuntimeOrigin::signed(admin.clone()),
		collection,
		item,
		*item_config,
		amount
	));
}

fn mint_items(collection: u32, amount: u32, count: u32) -> sr25519::Public {
	let who = new_account(10, 10_000_u128);

	for i in 0..count {
		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(who.clone()),
			collection,
			who.clone(),
			amount
		));
	}
	who
}

#[test]
fn create_first_game_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3_000 * unit(GAKI);
		let owner = new_account(0, before_balance);
		let admin = new_account(1, 3_000 * unit(GAKI));

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			admin.clone()
		));

		let game = Game::<Test>::get(0).unwrap();
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
fn create_game_collection_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let (owner, admin) = do_create_game();

		assert_ok!(PalletGame::create_game_collection(
			RuntimeOrigin::signed(admin.clone()),
			0,
			default_collection_config(),
		));

		assert_eq!(CollectionsOf::<Test>::get(0)[0], 0);
		assert_eq!(GameOf::<Test>::get(0), Some(0));

		assert_ok!(PalletGame::create_game_collection(
			RuntimeOrigin::signed(admin.clone()),
			0,
			default_collection_config(),
		));
		assert_eq!(CollectionsOf::<Test>::get(0)[1], 1);
	})
}

#[test]
fn create_game_collection_should_fails() {
	new_test_ext().execute_with(|| {
		// no permission
		{
			run_to_block(1);
			let (owner, admin) = do_create_game();

			let acc = new_account(3, 3 * unit(GAKI));
			// random acc should has no permission
			assert_err!(
				PalletGame::create_game_collection(
					RuntimeOrigin::signed(acc.clone()),
					0,
					default_collection_config(),
				),
				Error::<Test>::NoPermission
			);

			// game owner also should has no permission
			assert_err!(
				PalletGame::create_game_collection(
					RuntimeOrigin::signed(owner.clone()),
					0,
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
		let who = new_account(0, 3_000 * unit(GAKI));

		assert_ok!(PalletGame::create_collection(
			RuntimeOrigin::signed(who.clone()),
			who.clone(),
			default_collection_config(),
		));
	})
}

#[test]
fn add_game_collection_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let (_, admin) = do_create_game();

		do_create_collection(0, &default_collection_config(), &admin);
		do_create_collection(0, &default_collection_config(), &admin);

		assert_ok!(PalletGame::create_collection(
			RuntimeOrigin::signed(admin.clone()),
			admin.clone(),
			default_collection_config(),
		));

		assert_ok!(PalletGame::add_game_collection(
			RuntimeOrigin::signed(admin.clone()),
			0,
			2,
		));

		assert_eq!(CollectionsOf::<Test>::get(0), [0, 1, 2].to_vec());
		assert_eq!(GameOf::<Test>::get(0).unwrap(), 0);
		assert_eq!(GameOf::<Test>::get(1).unwrap(), 0);
		assert_eq!(GameOf::<Test>::get(2).unwrap(), 0);
	})
}

#[test]
fn add_game_collection_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let (owner, admin) = do_create_game();

		for i in 0..(MAX_GAME_COLLECTION_VAL) {
			assert_ok!(PalletGame::create_collection(
				RuntimeOrigin::signed(admin.clone()),
				admin.clone(),
				default_collection_config(),
			));

			assert_ok!(PalletGame::add_game_collection(
				RuntimeOrigin::signed(admin.clone()),
				0,
				i
			));
		}

		assert_err!(
			PalletGame::add_game_collection(RuntimeOrigin::signed(admin.clone()), 0, 0),
			Error::<Test>::CollectionExists
		);

		assert_ok!(PalletGame::create_collection(
			RuntimeOrigin::signed(admin.clone()),
			admin.clone(),
			default_collection_config(),
		));

		assert_err!(
			PalletGame::add_game_collection(RuntimeOrigin::signed(owner.clone()), 0, 1),
			Error::<Test>::NoPermission
		);

		assert_err!(
			PalletGame::add_game_collection(
				RuntimeOrigin::signed(admin.clone()),
				0,
				MAX_GAME_COLLECTION_VAL
			),
			Error::<Test>::ExceedMaxCollection
		);
	})
}

#[test]
fn create_item_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let (owner, admin) = do_create_game();
		do_create_collection(0, &default_collection_config(), &admin);

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
		let (owner, admin) = do_create_game();
		do_create_collection(0, &default_collection_config(), &admin);
		do_create_item(&admin, 0, 0, &default_item_config(), 1000);

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
		let (owner, admin) = do_create_game();
		let mint_fee = 1 * unit(GAKI);
		do_create_collection(0, &collection_config(mint_fee), &admin);
		do_create_item(&admin, 0, 0, &default_item_config(), 1000);

		let before_player_balance = 3000 * unit(GAKI);
		let before_owner_balance = Balances::free_balance(owner.clone());
		let player = new_account(2, before_player_balance);
		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			3
		));
		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 3);
		assert_eq!(
			Balances::free_balance(player.clone()),
			before_player_balance - (mint_fee * 3)
		);
		assert_eq!(
			Balances::free_balance(owner.clone()),
			before_owner_balance + (mint_fee * 3)
		);
	})
}

// #[test]
// fn mint_should_fails() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);
// 		let owner = new_account(0, 3 * unit(GAKI));

// 		let admin = new_account(1, 3 * unit(GAKI));
// 		let mint_fee: u128 = 3 * unit(GAKI);

// 		assert_ok!(PalletGame::create_game(
// 			RuntimeOrigin::signed(owner.clone()),
// 			admin.clone(),
// 		));

// 		assert_ok!(PalletGame::create_game_collection(
// 			RuntimeOrigin::signed(admin.clone()),
// 			0,
// 			admin.clone(),
// 			collection_config(mint_fee),
// 		));

// 		assert_ok!(PalletGame::create_item(
// 			RuntimeOrigin::signed(admin.clone()),
// 			0,
// 			0,
// 			default_item_config(),
// 			MAX_ITEM_MINT_VAL + 1
// 		));

// 		let player = new_account(2, 3000 * unit(GAKI));
// 		assert_err!(
// 			PalletGame::mint(
// 				RuntimeOrigin::signed(player.clone()),
// 				0,
// 				player.clone(),
// 				MAX_ITEM_MINT_VAL + 1
// 			),
// 			Error::<Test>::ExceedAllowedAmount
// 		);

// 		assert_ok!(PalletGame::mint(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 			player.clone(),
// 			MAX_ITEM_MINT_VAL,
// 		));

// 		// one left
// 		assert_err!(
// 			PalletGame::mint(RuntimeOrigin::signed(player.clone()), 0, player.clone(), 4),
// 			Error::<Test>::ExceedTotalAmount
// 		);

// 		assert_ok!(PalletGame::mint(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 			player.clone(),
// 			1
// 		));

// 		assert_err!(
// 			PalletGame::mint(RuntimeOrigin::signed(player.clone()), 0, player.clone(), 1),
// 			Error::<Test>::SoldOut
// 		);
// 	})
// }

// #[test]
// pub fn burn_items_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);
// 		let owner = new_account(0, 3 * unit(GAKI));

// 		assert_ok!(PalletGame::create_game(
// 			RuntimeOrigin::signed(owner.clone()),
// 			owner.clone()
// 		));

// 		assert_ok!(PalletGame::create_game_collection(
// 			RuntimeOrigin::signed(owner.clone()),
// 			0,
// 			owner.clone(),
// 			collection_config(0),
// 		));

// 		assert_ok!(PalletGame::create_item(
// 			RuntimeOrigin::signed(owner.clone()),
// 			0,
// 			0,
// 			default_item_config(),
// 			100
// 		));

// 		let player = new_account(2, 3000 * unit(GAKI));
// 		assert_ok!(PalletGame::mint(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 			player.clone(),
// 			10
// 		));
// 		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 10);

// 		assert_ok!(PalletGame::burn(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 			0,
// 			5
// 		));
// 		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 5);

// 		assert_err!(
// 			PalletGame::burn(RuntimeOrigin::signed(player.clone()), 0, 0, 6),
// 			Error::<Test>::InsufficientItemBalance
// 		);
// 	})
// }

// #[test]
// pub fn transfer_item_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let owner = new_account(0, 10_000 * unit(GAKI));

// 		create_item(
// 			&owner,
// 			&default_collection_config(),
// 			&default_item_config(),
// 			0,
// 			100,
// 		);

// 		let player = new_account(2, 3000 * unit(GAKI));
// 		let dest = new_account(3, 3000 * unit(GAKI));
// 		assert_ok!(PalletGame::mint(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 			player.clone(),
// 			10
// 		));

// 		assert_ok!(PalletGame::transfer(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 			0,
// 			dest.clone(),
// 			5
// 		));

// 		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 5);
// 		assert_eq!(ItemBalanceOf::<Test>::get((dest.clone(), 0, 0)), 5);

// 		assert_err!(
// 			PalletGame::transfer(RuntimeOrigin::signed(player.clone()), 0, 0, dest.clone(), 6),
// 			Error::<Test>::InsufficientItemBalance
// 		);
// 	})
// }

// #[test]
// pub fn set_upgrade_item_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let owner = new_account(0, 10_000 * unit(GAKI));

// 		create_item(
// 			&owner,
// 			&default_collection_config(),
// 			&default_item_config(),
// 			0,
// 			100,
// 		);

// 		let byte = 50;

// 		let input: UpgradeItemConfig<u32, u128> = UpgradeItemConfig {
// 			item: 100,
// 			fee: 3 * unit(GAKI),
// 		};

// 		let before_balance = Balances::free_balance(&owner);

// 		assert_ok!(PalletGame::set_upgrade_item(
// 			RuntimeOrigin::signed(owner.clone()),
// 			0,
// 			0,
// 			input.item,
// 			default_item_config(),
// 			bvec![0u8; byte],
// 			1,
// 			input.fee,
// 		));

// 		assert_eq!(LevelOf::<Test>::get(0, 0), 0);
// 		assert_eq!(LevelOf::<Test>::get(0, 100), 1);
// 		assert_eq!(OriginItemOf::<Test>::get((0, 100)).unwrap(), (0, 0));

// 		assert_eq!(UpgradeConfigOf::<Test>::get((0, 0, 1)).unwrap(), input);
// 		assert_eq!(
// 			Balances::free_balance(&owner),
// 			before_balance -
// 				UPGRADE_DEPOSIT_VAL -
// 				ITEM_DEPOSIT_VAL - (BYTE_DEPOSIT_VAL * u128::try_from(byte).unwrap())
// 		);
// 	})
// }

// #[test]
// pub fn upgrade_item_shoud_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let owner = new_account(0, 10_000 * unit(GAKI));

// 		create_item(
// 			&owner,
// 			&default_collection_config(),
// 			&default_item_config(),
// 			0,
// 			100,
// 		);

// 		let byte = 50;

// 		let input: UpgradeItemConfig<u32, u128> = UpgradeItemConfig {
// 			item: 100,
// 			fee: 3 * unit(GAKI),
// 		};

// 		assert_ok!(PalletGame::set_upgrade_item(
// 			RuntimeOrigin::signed(owner.clone()),
// 			0,
// 			0,
// 			input.item,
// 			default_item_config(),
// 			bvec![0u8; byte],
// 			1,
// 			input.fee,
// 		));

// 		let player = new_account(3, 10000 * unit(GAKI));

// 		assert_ok!(PalletGame::mint(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 			player.clone(),
// 			10
// 		));

// 		let player_before_balance = Balances::free_balance(&player);
// 		let owner_before_balance = Balances::free_balance(&owner);

// 		assert_ok!(PalletGame::upgrade_item(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 			0,
// 			3
// 		));

// 		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 7);
// 		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 100)), 3);
// 		assert_eq!(
// 			Balances::free_balance(&player),
// 			player_before_balance - (input.fee * 3)
// 		);
// 		assert_eq!(
// 			Balances::free_balance(&owner),
// 			owner_before_balance + (input.fee * 3)
// 		);
// 	})
// }

// #[test]
// pub fn set_price_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let owner = new_account(0, 10_000 * unit(GAKI));

// 		create_item(
// 			&owner,
// 			&default_collection_config(),
// 			&default_item_config(),
// 			0,
// 			100,
// 		);

// 		let player = new_account(3, 10000 * unit(GAKI));

// 		assert_ok!(PalletGame::mint(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 			player.clone(),
// 			10
// 		));

// 		let package = Package {
// 			collection: 0,
// 			item: 0,
// 			amount: 9,
// 		};
// 		let price = 3 * unit(GAKI);

// 		let before_balance = Balances::free_balance(&player);

// 		assert_ok!(PalletGame::set_price(
// 			RuntimeOrigin::signed(player.clone()),
// 			package.clone(),
// 			price,
// 		));
// 		assert_eq!(
// 			Balances::free_balance(&player),
// 			before_balance - SALE_DEPOSIT_VAL
// 		);
// 		assert_eq!(PackageOf::<Test>::get(0).unwrap(), package);
// 		assert_eq!(
// 			TradeConfigOf::<Test>::get(0).unwrap(),
// 			TradeConfig {
// 				trade: TradeType::Normal,
// 				owner: player.clone(),
// 				price,
// 			}
// 		);
// 	})
// }

// #[test]
// pub fn set_price_should_fails() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let owner = new_account(0, 10_000 * unit(GAKI));

// 		let mut item_config = ItemConfig::default();
// 		item_config.disable_setting(pallet_nfts::ItemSetting::Transferable);

// 		create_item(&owner, &default_collection_config(), &item_config, 0, 100);

// 		let player = new_account(3, 10000 * unit(GAKI));

// 		assert_ok!(PalletGame::mint(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 			player.clone(),
// 			10
// 		));

// 		let mut package = Package {
// 			collection: 0,
// 			item: 0,
// 			amount: 11,
// 		};
// 		let price = 3 * unit(GAKI);
// 		assert_err!(
// 			PalletGame::set_price(
// 				RuntimeOrigin::signed(player.clone()),
// 				package.clone(),
// 				price,
// 			),
// 			Error::<Test>::InsufficientItemBalance
// 		);

// 		package.amount = 1;
// 		assert_err!(
// 			PalletGame::set_price(RuntimeOrigin::signed(player.clone()), package, price,),
// 			Error::<Test>::ItemLocked
// 		);
// 	})
// }

// #[test]
// pub fn buy_item_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let owner = new_account(0, 10_000 * unit(GAKI));

// 		create_item(
// 			&owner,
// 			&default_collection_config(),
// 			&default_item_config(),
// 			0,
// 			100,
// 		);

// 		let seller = new_account(3, 10000 * unit(GAKI));

// 		assert_ok!(PalletGame::mint(
// 			RuntimeOrigin::signed(seller.clone()),
// 			0,
// 			seller.clone(),
// 			10
// 		));

// 		let price = 3 * unit(GAKI);
// 		let package = Package {
// 			collection: 0,
// 			item: 0,
// 			amount: 8,
// 		};

// 		assert_ok!(PalletGame::set_price(
// 			RuntimeOrigin::signed(seller.clone()),
// 			package,
// 			price,
// 		));

// 		let buyer = new_account(4, 10000 * unit(GAKI));

// 		let seller_before_balance = Balances::free_balance(&seller);
// 		let buyer_before_balance = Balances::free_balance(&buyer);

// 		assert_ok!(PalletGame::buy_item(
// 			RuntimeOrigin::signed(buyer.clone()),
// 			0,
// 			3,
// 			price,
// 		));

// 		assert_eq!(ItemBalanceOf::<Test>::get((&seller, 0, 0)), 2);
// 		assert_eq!(LockBalanceOf::<Test>::get((&seller, 0, 0)), 5);
// 		assert_eq!(ItemBalanceOf::<Test>::get((&buyer, 0, 0)), 3);

// 		assert_eq!(
// 			Balances::free_balance(&seller),
// 			seller_before_balance + (price * 3)
// 		);
// 		assert_eq!(
// 			Balances::free_balance(&buyer),
// 			buyer_before_balance - (price * 3)
// 		);
// 	})
// }

// #[test]
// pub fn buy_item_should_fails() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let owner = new_account(0, 10_000 * unit(GAKI));

// 		create_item(
// 			&owner,
// 			&default_collection_config(),
// 			&default_item_config(),
// 			0,
// 			100,
// 		);

// 		let seller = new_account(3, 10000 * unit(GAKI));

// 		assert_ok!(PalletGame::mint(
// 			RuntimeOrigin::signed(seller.clone()),
// 			0,
// 			seller.clone(),
// 			10
// 		));

// 		let price = 3 * unit(GAKI);
// 		let package = Package {
// 			collection: 0,
// 			item: 0,
// 			amount: 10,
// 		};

// 		assert_ok!(PalletGame::set_price(
// 			RuntimeOrigin::signed(seller.clone()),
// 			package,
// 			price,
// 		));

// 		let buyer = new_account(4, 10000 * unit(GAKI));

// 		assert_err!(
// 			PalletGame::buy_item(
// 				RuntimeOrigin::signed(buyer.clone()),
// 				0,
// 				1,
// 				price - (1 * unit(GAKI)),
// 			),
// 			Error::<Test>::BidTooLow
// 		);

// 		assert_ok!(PalletGame::buy_item(
// 			RuntimeOrigin::signed(buyer.clone()),
// 			0,
// 			8,
// 			price,
// 		));

// 		assert_err!(
// 			PalletGame::buy_item(RuntimeOrigin::signed(buyer.clone()), 0, 4, price,),
// 			Error::<Test>::SoldOut
// 		);
// 	})
// }

// #[test]
// pub fn set_bundle_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let count = 2_u32;
// 		let seller = new_account(0, 10_000 * unit(GAKI));

// 		mint_items(&seller, 10, count);

// 		let mut packages: Vec<PackageFor<Test>> = vec![];
// 		for i in 0..count {
// 			packages.push(Package::new(0, i, 10));
// 		}

// 		let price = 100 * unit(GAKI);

// 		assert_ok!(PalletGame::set_bundle(
// 			RuntimeOrigin::signed(seller.clone()),
// 			packages.clone(),
// 			price
// 		));

// 		assert_eq!(BundleOf::<Test>::get(0), packages);
// 	})
// }

// #[test]
// pub fn set_bundle_should_fails() {
// 	new_test_ext().execute_with(|| {
// 		let seller = new_account(0, 10_000 * unit(GAKI));

// 		for i in 0..(MAX_BUNDLE_VAL + 5) {
// 			ItemBalanceOf::<Test>::insert((&seller, 0, i), 10);
// 		}

// 		{
// 			let packages: Vec<PackageFor<Test>> = vec![Package::new(0, 0, 11)];
// 			assert_err!(
// 				PalletGame::set_bundle(
// 					RuntimeOrigin::signed(seller.clone()),
// 					packages.clone(),
// 					100 * unit(GAKI),
// 				),
// 				Error::<Test>::InsufficientItemBalance
// 			);
// 		}

// 		{
// 			let mut packages: Vec<PackageFor<Test>> = vec![];

// 			for i in 0..(MAX_BUNDLE_VAL + 5) {
// 				packages.push(Package::new(0, i, 1));
// 			}
// 			assert_err!(
// 				PalletGame::set_bundle(
// 					RuntimeOrigin::signed(seller.clone()),
// 					packages.clone(),
// 					100 * unit(GAKI),
// 				),
// 				Error::<Test>::ExceedMaxBundle
// 			);
// 		}
// 	})
// }

// #[test]
// pub fn buy_bundle_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let count = 2_u32;
// 		let seller = new_account(0, 10_000 * unit(GAKI));

// 		mint_items(&seller, 10, count);

// 		let mut packages: Vec<PackageFor<Test>> = vec![];
// 		for i in 0..count {
// 			packages.push(Package::new(0, i, 5));
// 		}

// 		let price = 100 * unit(GAKI);

// 		assert_ok!(PalletGame::set_bundle(
// 			RuntimeOrigin::signed(seller.clone()),
// 			packages.clone(),
// 			price
// 		));

// 		let buyer = new_account(1, 10_000 * unit(GAKI));

// 		let seller_before_balance = Balances::free_balance(&seller);
// 		let buyer_before_balance = Balances::free_balance(&buyer);

// 		assert_ok!(PalletGame::buy_bundle(
// 			RuntimeOrigin::signed(buyer.clone()),
// 			0,
// 			price
// 		));
// 		for i in 0..count {
// 			assert_eq!(ItemBalanceOf::<Test>::get((buyer.clone(), 0, i)), 5);
// 		}
// 		for i in 0..count {
// 			assert_eq!(ItemBalanceOf::<Test>::get((seller.clone(), 0, i)), 5);
// 		}
// 		assert_eq!(
// 			Balances::free_balance(&seller),
// 			seller_before_balance + price + BUNDLE_DEPOSIT_VAL
// 		);
// 		assert_eq!(Balances::free_balance(&buyer), buyer_before_balance - price);
// 	})
// }

// #[test]
// pub fn buy_bundle_should_fails() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let count = 2_u32;
// 		let seller = new_account(0, 10_000 * unit(GAKI));

// 		mint_items(&seller, 10, count);

// 		let mut packages: Vec<PackageFor<Test>> = vec![];
// 		for i in 0..count {
// 			packages.push(Package::new(0, i, 5));
// 		}

// 		let price = 100 * unit(GAKI);

// 		assert_ok!(PalletGame::set_bundle(
// 			RuntimeOrigin::signed(seller.clone()),
// 			packages.clone(),
// 			price
// 		));

// 		let buyer = new_account(1, 1 * unit(GAKI));
// 		assert_err!(
// 			PalletGame::buy_bundle(RuntimeOrigin::signed(buyer.clone()), 0, price * unit(GAKI)),
// 			pallet_balances::Error::<Test>::InsufficientBalance
// 		);

// 		let buyer = new_account(1, 1000 * unit(GAKI));
// 		assert_err!(
// 			PalletGame::buy_bundle(
// 				RuntimeOrigin::signed(buyer.clone()),
// 				0,
// 				price - 1 * unit(GAKI)
// 			),
// 			Error::<Test>::BidTooLow
// 		);
// 	})
// }

// #[test]
// pub fn cancel_set_price_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let count = 2_u32;
// 		let player = new_account(2, 10_000 * unit(GAKI));
// 		mint_items(&player, 10, count);

// 		let package = Package {
// 			collection: 0,
// 			item: 0,
// 			amount: 10,
// 		};

// 		assert_ok!(PalletGame::set_price(
// 			RuntimeOrigin::signed(player.clone()),
// 			package,
// 			3 * unit(GAKI),
// 		));

// 		let before_balance = Balances::free_balance(&player);

// 		assert_ok!(PalletGame::cancel_set_price(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 		));

// 		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 10);
// 		assert_eq!(LockBalanceOf::<Test>::get((player.clone(), 0, 0)), 0);
// 		assert_eq!(
// 			Balances::free_balance(&player),
// 			before_balance + SALE_DEPOSIT_VAL
// 		);
// 	});
// }

// #[test]
// pub fn cancel_set_bundle_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let count = 2_u32;
// 		let player = new_account(2, 10_000 * unit(GAKI));
// 		mint_items(&player, 10, count);

// 		let mut packages: Vec<PackageFor<Test>> = vec![];
// 		for i in 0..count {
// 			packages.push(Package::new(0, i, 5));
// 		}

// 		assert_ok!(PalletGame::set_bundle(
// 			RuntimeOrigin::signed(player.clone()),
// 			packages,
// 			3 * unit(GAKI),
// 		));

// 		let before_balance = Balances::free_balance(&player);

// 		assert_ok!(PalletGame::cancel_set_bundle(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 		));

// 		for i in 0..count {
// 			assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, i)), 10);
// 			assert_eq!(LockBalanceOf::<Test>::get((player.clone(), 0, i)), 0);
// 		}
// 		assert_eq!(
// 			Balances::free_balance(&player),
// 			before_balance + BUNDLE_DEPOSIT_VAL
// 		);
// 	});
// }

// #[test]
// pub fn set_wishlist_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);
// 		let count = 2_u32;
// 		let buyer = new_account(3, 1000 * unit(GAKI));
// 		let mut packages: Vec<PackageFor<Test>> = vec![];
// 		for i in 0..count {
// 			packages.push(Package::new(0, i, 5));
// 		}
// 		let price = 3 * unit(GAKI);

// 		let before_balance = Balances::free_balance(&buyer);
// 		assert_ok!(PalletGame::set_wishlist(
// 			RuntimeOrigin::signed(buyer.clone()),
// 			packages.clone(),
// 			price,
// 		));

// 		assert_eq!(BundleOf::<Test>::get(0), packages);
// 		assert_eq!(
// 			TradeConfigOf::<Test>::get(0).unwrap(),
// 			TradeConfig {
// 				trade: TradeType::Wishlist,
// 				owner: buyer.clone(),
// 				price
// 			}
// 		);
// 		assert_eq!(
// 			Balances::free_balance(&buyer),
// 			before_balance - (BUNDLE_DEPOSIT_VAL + price)
// 		);
// 	})
// }

// #[test]
// pub fn fill_wishlist_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);
// 		let count = 2_u32;
// 		let player = new_account(2, 10_000 * unit(GAKI));
// 		mint_items(&player, 10, count);

// 		let mut packages: Vec<PackageFor<Test>> = vec![];
// 		for i in 0..count {
// 			packages.push(Package::new(0, i, 5));
// 		}
// 		let price = 3 * unit(GAKI);

// 		let buyer = new_account(3, 1000 * unit(GAKI));
// 		assert_ok!(PalletGame::set_wishlist(
// 			RuntimeOrigin::signed(buyer.clone()),
// 			packages.clone(),
// 			price,
// 		));

// 		let before_player_balance = Balances::free_balance(&player);
// 		assert_ok!(PalletGame::fill_wishlist(
// 			RuntimeOrigin::signed(player.clone()),
// 			0,
// 			price
// 		));

// 		for i in 0..count {
// 			assert_eq!(ItemBalanceOf::<Test>::get((&player, 0, i)), 5);
// 			assert_eq!(ItemBalanceOf::<Test>::get((&buyer, 0, i)), 5);
// 		}
// 		assert_eq!(Balances::reserved_balance(&buyer), 0);
// 		assert_eq!(
// 			Balances::free_balance(&player),
// 			before_player_balance + price
// 		);
// 	})
// }
