use crate::{mock::*, types::*, Error, *};
use sp_core::sr25519;

use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_support::{
	common::{unit, NativeToken::GAKI},
	game::{Loot, Package, NFT},
};
use pallet_nfts::{CollectionRole, CollectionRoles};
use sp_keystore::{testing::MemoryKeystore, KeystoreExt};
use sp_runtime::TokenError;

type PackageFor<T> =
	Package<<T as pallet_nfts::Config>::CollectionId, <T as pallet_nfts::Config>::ItemId>;

type LootFor<T> =
	Loot<<T as pallet_nfts::Config>::CollectionId, <T as pallet_nfts::Config>::ItemId>;

fn make_deposit(account: &sr25519::Public, balance: u128) {
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(account: u32, balance: u128) -> sr25519::Public {
	let keystore = KeystoreExt::new(MemoryKeystore::new());
	let acc: sr25519::Public = keystore
		.sr25519_generate_new(sp_runtime::KeyTypeId::from(account), None)
		.unwrap();
	make_deposit(&acc, balance);
	return acc
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

fn do_create_collection(game: u32, admin: &sr25519::Public) {
	assert_ok!(PalletGame::create_game_collection(
		RuntimeOrigin::signed(admin.clone()),
		game,
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
		Some(amount)
	));
}

const TEST_BUNDLE: [PackageFor<Test>; 3] = [
	Package {
		collection: 0,
		item: 0,
		amount: 10,
	},
	Package {
		collection: 0,
		item: 1,
		amount: 10,
	},
	Package {
		collection: 0,
		item: 2,
		amount: 10,
	},
];

const TEST_TABLE: [LootFor<Test>; 3] = [
	Loot {
		maybe_nft: Some(NFT {
			collection: 0,
			item: 0,
		}),
		weight: 30000, // 30%
	},
	Loot {
		maybe_nft: Some(NFT {
			collection: 0,
			item: 1,
		}),
		weight: 35000, // 30%
	},
	Loot {
		maybe_nft: Some(NFT {
			collection: 0,
			item: 2,
		}),
		weight: 35000, // 30%
	},
];

const TEST_BUNDLE1: [PackageFor<Test>; 3] = [
	Package {
		collection: 1,
		item: 0,
		amount: 10,
	},
	Package {
		collection: 1,
		item: 1,
		amount: 10,
	},
	Package {
		collection: 1,
		item: 2,
		amount: 10,
	},
];

fn do_mint_item(collection: u32, amount: u32) -> sr25519::Public {
	let player = new_account(10, 100_000 * unit(GAKI));
	assert_ok!(PalletGame::mint(
		RuntimeOrigin::signed(player.clone()),
		collection,
		player.clone(),
		amount
	));
	player
}

// fn do_all_create_dynamic_pool(
// 	fee: u128,
// 	source: [LootFor<Test>; 3],
// ) -> (sr25519::Public, sr25519::Public) {
// 	let (owner, admin) = do_create_game();
// 	let latest_id = NextGameId::<Test>::get().unwrap() - 1;
// 	do_create_collection(latest_id, &admin);

// 	for pack in source.clone() {
// 		do_create_item(
// 			&admin,
// 			latest_id,
// 			pack.unwrap().item,
// 			&default_item_config(),
// 			pack.weight,
// 		);
// 	}
// 	do_create_dynamic_pool(&owner, &admin, fee, source);
// 	(owner, admin)
// }

// fn do_all_create_stable_pool(
// 	fee: u128,
// 	dist: Vec<LootFor<Test>>,
// ) -> (sr25519::Public, sr25519::Public) {
// 	let (owner, admin) = do_create_game();
// 	let latest_id = NextGameId::<Test>::get().unwrap() - 1;
// 	do_create_collection(latest_id, &admin);

// 	for pack in dist.clone() {
// 		assert_ok!(PalletGame::create_item(
// 			RuntimeOrigin::signed(admin.clone()),
// 			latest_id,
// 			pack.unwrap().item,
// 			default_item_config(),
// 			None
// 		));
// 	}
// 	assert_ok!(PalletGame::create_stable_pool(
// 		RuntimeOrigin::signed(owner.clone()),
// 		dist.to_vec(),
// 		fee,
// 		admin.clone(),
// 	));
// 	(owner, admin)
// }

fn do_create_dynamic_pool(
	owner: &sr25519::Public,
	admin: &sr25519::Public,
	fee: u128,
	source: [LootFor<Test>; 3],
) {
	assert_ok!(PalletGame::create_dynamic_pool(
		RuntimeOrigin::signed(owner.clone()),
		source.to_vec(),
		fee,
		admin.clone(),
	));
}

fn create_account_with_item(source: [PackageFor<Test>; 3]) -> sr25519::Public {
	let (owner, admin) = do_create_game();
	let latest_game = NextGameId::<Test>::get().unwrap() - 1;

	do_create_collection(latest_game, &admin);

	for pack in source.clone() {
		do_create_item(
			&admin,
			pack.collection,
			pack.item,
			&default_item_config(),
			u32::MAX,
		);
	}

	let player = new_account(3, 1000 * unit(GAKI));
	for pack in source.clone() {
		assert_ok!(PalletGame::transfer(
			RuntimeOrigin::signed(owner.clone()),
			pack.collection,
			pack.item,
			player.clone(),
			pack.amount
		));
	}
	player
}

fn do_all_set_price(package: PackageFor<Test>, price: u128) -> sr25519::Public {
	let who = create_account_with_item(TEST_BUNDLE);

	assert_ok!(PalletGame::set_price(
		RuntimeOrigin::signed(who.clone()),
		package.clone(),
		price,
	));
	who
}

fn do_all_set_bundle(package: Vec<PackageFor<Test>>, price: u128) -> sr25519::Public {
	let player = create_account_with_item(TEST_BUNDLE);

	assert_ok!(PalletGame::set_bundle(
		RuntimeOrigin::signed(player.clone()),
		package,
		price
	));
	player
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
		));

		assert_eq!(CollectionsOf::<Test>::get(0)[0], 0);
		assert_eq!(GamesOf::<Test>::get(0), [0].to_vec());

		assert_ok!(PalletGame::create_game_collection(
			RuntimeOrigin::signed(admin.clone()),
			0,
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
				PalletGame::create_game_collection(RuntimeOrigin::signed(acc.clone()), 0,),
				Error::<Test>::NoPermission
			);

			// game owner also should has no permission
			assert_err!(
				PalletGame::create_game_collection(RuntimeOrigin::signed(owner.clone()), 0,),
				Error::<Test>::NoPermission
			);
		}
	})
}

#[test]
fn remove_collection_should_works() {
	new_test_ext().execute_with(|| {
		let (owner, admin) = do_create_game();

		do_create_collection(0, &admin);
		do_create_collection(0, &admin);
		do_create_collection(0, &admin);

		assert_ok!(PalletGame::remove_collection(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0
		));

		let new_admin = new_account(3, 1000);
		assert_ok!(PalletGame::set_team(
			RuntimeOrigin::signed(owner.clone()),
			1,
			None,
			Some(new_admin),
			None
		));

		assert_ok!(PalletGame::remove_collection(
			RuntimeOrigin::signed(new_admin.clone()),
			0,
			1
		));

		assert_eq!(GamesOf::<Test>::get(0), [].to_vec());
		assert_eq!(GamesOf::<Test>::get(1), [].to_vec());
		assert_eq!(CollectionsOf::<Test>::get(0), [2].to_vec());
	})
}

#[test]
fn remove_collection_should_fails() {
	new_test_ext().execute_with(|| {
		let (owner, admin) = do_create_game();
		do_create_collection(0, &admin);
		do_create_collection(0, &admin);
		do_create_collection(0, &admin);

		assert_err!(
			PalletGame::remove_collection(RuntimeOrigin::signed(owner.clone()), 0, 1),
			Error::<Test>::NoPermission
		);
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
		));
	})
}

#[test]
fn set_accept_adding_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let who = new_account(0, 3_000 * unit(GAKI));
		assert_ok!(PalletGame::create_collection(
			RuntimeOrigin::signed(who.clone()),
			who.clone(),
		));

		assert_ok!(PalletGame::set_accept_adding(
			RuntimeOrigin::signed(who.clone()),
			0,
			0
		));
		assert_err!(
			PalletGame::set_accept_adding(RuntimeOrigin::signed(who.clone()), 0, 1),
			Error::<Test>::UnknownCollection
		);
	})
}

#[test]
fn add_game_collection_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let (owner, admin) = do_create_game();

		let who = new_account(1, 1000 * unit(GAKI));

		assert_ok!(PalletGame::create_collection(
			RuntimeOrigin::signed(who.clone()),
			who.clone(),
		));

		assert_ok!(PalletGame::set_accept_adding(
			RuntimeOrigin::signed(who.clone()),
			0,
			0
		));

		assert_ok!(PalletGame::add_game_collection(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0,
		));

		assert_eq!(CollectionsOf::<Test>::get(0), [0].to_vec());
		assert_eq!(GamesOf::<Test>::get(0), [0].to_vec());
	})
}

#[test]
fn add_game_collection_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let (owner, admin) = do_create_game();

		for i in 0..(MAX_GAME_COLLECTION_VAL) {
			assert_ok!(PalletGame::create_collection(
				RuntimeOrigin::signed(owner.clone()),
				admin.clone(),
			));

			assert_ok!(PalletGame::set_accept_adding(
				RuntimeOrigin::signed(admin),
				0,
				i
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
		));

		assert_err!(
			PalletGame::add_game_collection(RuntimeOrigin::signed(owner.clone()), 0, 1),
			Error::<Test>::NoPermission
		);

		assert_ok!(PalletGame::set_accept_adding(
			RuntimeOrigin::signed(admin),
			0,
			MAX_GAME_COLLECTION_VAL
		));

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
		do_create_collection(0, &admin);

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0,
			default_item_config(),
			Some(1000)
		));

		assert_eq!(ItemBalanceOf::<Test>::get((owner.clone(), 0, 0)), 1000);
	})
}

// #[test]
// fn add_item_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);
// 		let (owner, admin) = do_create_game();
// 		do_create_collection(0, &admin);
// 		do_create_item(&admin, 0, 0, &default_item_config(), 1000);

// 		assert_ok!(PalletGame::add_item(
// 			RuntimeOrigin::signed(admin.clone()),
// 			0,
// 			0,
// 			1000
// 		));

// 		assert_eq!(ItemBalanceOf::<Test>::get((owner.clone(), 0, 0)), 2000);
// 	})
// }

// #[test]
// fn create_dynamic_pool_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);
// 		let (owner, admin) = do_create_game();
// 		do_create_collection(0, &admin);

// 		for package in TEST_TABLE.clone() {
// 			do_create_item(
// 				&admin,
// 				package.maybe_nft.unwrap().collection,
// 				package.maybe_nft.unwrap().item,
// 				&default_item_config(),
// 				package.weight,
// 			);
// 		}

// 		let owner_balance = Balances::free_balance(owner.clone());

// 		assert_ok!(PalletGame::create_dynamic_pool(
// 			RuntimeOrigin::signed(owner.clone()),
// 			TEST_TABLE.clone().to_vec(),
// 			10 * unit(GAKI),
// 			admin.clone()
// 		));

// 		for package in TEST_TABLE.clone() {
// 			assert_eq!(
// 				ItemBalanceOf::<Test>::get((owner.clone(), package.maybe_nft.unwrap().collection, package.item)),
// 				0
// 			);

// 			assert_eq!(
// 				ReservedBalanceOf::<Test>::get((owner.clone(), package.collection, package.item)),
// 				package.amount
// 			);

// 			assert_eq!(LootTableOf::<Test>::get(0).to_vec(), TEST_TABLE.clone());
// 		}

// 		assert_eq!(
// 			Balances::free_balance(owner.clone()),
// 			owner_balance - MINING_DEPOSIT_VAL
// 		);
// 	})
// }

// #[test]
// fn mint_dynamic_pool_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);
// 		let mint_fee = 2 * unit(GAKI);
// 		let (owner, _) = do_all_create_dynamic_pool(mint_fee, TEST_BUNDLE.clone());
// 		let player = new_account(2, 1000_000 * unit(GAKI));
// 		// Independent collection
// 		{
// 			let owner_balance = Balances::free_balance(owner.clone());
// 			let player_balance = Balances::free_balance(player.clone());

// 			let amount = 10;
// 			for _ in TEST_BUNDLE.clone() {
// 				assert_ok!(PalletGame::mint(
// 					RuntimeOrigin::signed(player.clone()),
// 					0,
// 					player.clone(),
// 					amount,
// 				));
// 			}

// 			for pack in TEST_BUNDLE.clone() {
// 				assert_eq!(
// 					ItemBalanceOf::<Test>::get((player.clone(), pack.collection, pack.item)),
// 					amount
// 				);
// 				assert_eq!(
// 					ReservedBalanceOf::<Test>::get((owner.clone(), pack.collection, pack.item)),
// 					pack.amount - amount
// 				);
// 			}
// 			assert_eq!(
// 				Balances::free_balance(player.clone()),
// 				player_balance - (mint_fee * amount as u128 * TEST_BUNDLE.len() as u128)
// 			);
// 			assert_eq!(
// 				Balances::free_balance(owner.clone()),
// 				owner_balance + (mint_fee * amount as u128 * TEST_BUNDLE.len() as u128)
// 			);
// 		}
// 	})
// }

// #[test]
// fn mint_dynamic_pool_should_fails() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);
// 		let (_, _) = do_all_create_dynamic_pool(1 * unit(GAKI), TEST_BUNDLE);

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

// 		let total = PalletGame::total_weight(&TEST_TABLE.clone().to_vec());
// 		assert_err!(
// 			PalletGame::mint(
// 				RuntimeOrigin::signed(player.clone()),
// 				0,
// 				player.clone(),
// 				total + 1
// 			),
// 			Error::<Test>::ExceedTotalAmount
// 		);

// 		for package in TEST_TABLE.clone() {
// 			assert_ok!(PalletGame::mint(
// 				RuntimeOrigin::signed(player.clone()),
// 				0,
// 				player.clone(),
// 				package.amount,
// 			));
// 		}

// 		assert_err!(
// 			PalletGame::mint(RuntimeOrigin::signed(player.clone()), 0, player.clone(), 1),
// 			Error::<Test>::SoldOut
// 		);
// 	})
// }

#[test]
pub fn burn_items_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let player = create_account_with_item(TEST_BUNDLE);

		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 10);

		assert_ok!(PalletGame::burn(
			RuntimeOrigin::signed(player.clone()),
			0,
			0,
			5
		));
		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 5);

		assert_err!(
			PalletGame::burn(RuntimeOrigin::signed(player.clone()), 0, 0, 6),
			Error::<Test>::InsufficientItemBalance
		);
	})
}

#[test]
pub fn transfer_item_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let player = create_account_with_item(TEST_BUNDLE);

		let dest = new_account(3, 3000 * unit(GAKI));
		assert_ok!(PalletGame::transfer(
			RuntimeOrigin::signed(player.clone()),
			0,
			0,
			dest.clone(),
			5
		));

		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 5);
		assert_eq!(ItemBalanceOf::<Test>::get((dest.clone(), 0, 0)), 5);

		assert_err!(
			PalletGame::transfer(RuntimeOrigin::signed(player.clone()), 0, 0, dest.clone(), 6),
			Error::<Test>::InsufficientItemBalance
		);
	})
}

#[test]
pub fn set_upgrade_item_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let (owner, admin) = do_create_game();
		let mint_fee = 1 * unit(GAKI);
		do_create_collection(0, &admin);
		do_create_item(&admin, 0, 0, &default_item_config(), 1000);

		let byte = 50;

		let input: UpgradeItemConfig<u32, u128> = UpgradeItemConfig {
			item: 100,
			fee: 3 * unit(GAKI),
		};

		let before_balance = Balances::free_balance(&owner);

		assert_ok!(PalletGame::set_upgrade_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0,
			input.item,
			default_item_config(),
			bvec![0u8; byte],
			1,
			input.fee,
		));

		assert_eq!(LevelOf::<Test>::get(0, 0), 0);
		assert_eq!(LevelOf::<Test>::get(0, 100), 1);
		assert_eq!(OriginItemOf::<Test>::get((0, 100)).unwrap(), (0, 0));

		assert_eq!(UpgradeConfigOf::<Test>::get((0, 0, 1)).unwrap(), input);
		assert_eq!(
			Balances::free_balance(&owner),
			before_balance -
				UPGRADE_DEPOSIT_VAL -
				ITEM_DEPOSIT_VAL - (BYTE_DEPOSIT_VAL * u128::try_from(byte).unwrap())
		);
	})
}

// #[test]
// pub fn upgrade_item_shoud_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);
// 		let (owner, admin) = do_create_game();
// 		let mint_fee = 1 * unit(GAKI);
// 		do_create_collection(0, &admin);
// 		do_create_item(&admin, 0, 0, &default_item_config(), 1000);
// 		let player = do_mint_item(0, 10);

// 		let byte = 50;

// 		let input: UpgradeItemConfig<u32, u128> = UpgradeItemConfig {
// 			item: 100,
// 			fee: 3 * unit(GAKI),
// 		};

// 		assert_ok!(PalletGame::set_upgrade_item(
// 			RuntimeOrigin::signed(admin.clone()),
// 			0,
// 			0,
// 			input.item,
// 			default_item_config(),
// 			bvec![0u8; byte],
// 			1,
// 			input.fee,
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

#[test]
pub fn set_price_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let player = create_account_with_item(TEST_BUNDLE);
		let price = 3 * unit(GAKI);

		let before_balance = Balances::free_balance(&player);
		assert_ok!(PalletGame::set_price(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE[0].clone(),
			price,
		));
		assert_eq!(
			Balances::free_balance(&player),
			before_balance - BUNDLE_DEPOSIT_VAL
		);
		assert_eq!(BundleOf::<Test>::get(0), [TEST_BUNDLE[0].clone()].to_vec());
		assert_eq!(
			TradeConfigOf::<Test>::get(0).unwrap(),
			TradeConfig {
				trade: TradeType::SetPrice,
				owner: player.clone(),
				maybe_price: Some(price),
				maybe_required: None,
			}
		);
	})
}

// #[test]
// pub fn set_price_should_fails() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let (owner, admin) = do_create_game();
// 		let mint_fee = 1 * unit(GAKI);
// 		do_create_collection(0, &admin);
// 		do_create_item(&admin, 0, 0, &default_item_config(), 1000);
// 		let player = do_mint_item(0, 10);

// 		let price = 3 * unit(GAKI);

// 		let mut fail_bundle = TEST_BUNDLE[0].clone();
// 		fail_bundle.amount += 10;
// 		assert_err!(
// 			PalletGame::set_price(
// 				RuntimeOrigin::signed(player.clone()),
// 				fail_bundle.clone(),
// 				price,
// 			),
// 			Error::<Test>::InsufficientItemBalance
// 		);

// 		let mut item_config = ItemConfig::default();
// 		item_config.disable_setting(pallet_nfts::ItemSetting::Transferable);

// 		assert_ok!(pallet_nfts::pallet::Pallet::<Test>::lock_item_transfer(
// 			RuntimeOrigin::signed(admin),
// 			0,
// 			0
// 		));

// 		assert_err!(
// 			PalletGame::set_price(
// 				RuntimeOrigin::signed(player.clone()),
// 				TEST_BUNDLE[0].clone(),
// 				price,
// 			),
// 			Error::<Test>::ItemLocked
// 		);
// 	})
// }

#[test]
pub fn buy_item_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let price = 1 * unit(GAKI);
		let seller = do_all_set_price(TEST_BUNDLE[0].clone(), price);

		let buyer = new_account(4, 10000 * unit(GAKI));

		let seller_before_balance = Balances::free_balance(&seller);
		let buyer_before_balance = Balances::free_balance(&buyer);

		assert_ok!(PalletGame::buy_item(
			RuntimeOrigin::signed(buyer.clone()),
			0,
			5,
			price,
		));

		assert_eq!(ItemBalanceOf::<Test>::get((&seller, 0, 0)), 0);
		assert_eq!(ReservedBalanceOf::<Test>::get((&seller, 0, 0)), 5);
		assert_eq!(ItemBalanceOf::<Test>::get((&buyer, 0, 0)), 5);

		assert_eq!(
			Balances::free_balance(&seller),
			seller_before_balance + (price * 5)
		);
		assert_eq!(
			Balances::free_balance(&buyer),
			buyer_before_balance - (price * 5)
		);
	})
}

#[test]
pub fn buy_item_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let price = 1 * unit(GAKI);
		do_all_set_price(TEST_BUNDLE[0].clone(), price);

		let buyer = new_account(4, 10000 * unit(GAKI));
		assert_err!(
			PalletGame::buy_item(
				RuntimeOrigin::signed(buyer.clone()),
				0,
				1,
				price - (1 * unit(GAKI)),
			),
			Error::<Test>::BidTooLow
		);

		assert_ok!(PalletGame::buy_item(
			RuntimeOrigin::signed(buyer.clone()),
			0,
			8,
			price,
		));

		assert_err!(
			PalletGame::buy_item(RuntimeOrigin::signed(buyer.clone()), 0, 4, price),
			Error::<Test>::SoldOut
		);
	})
}

#[test]
pub fn add_retail_supply_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let total = TEST_BUNDLE[0].clone();
		let supply = Package {
			collection: total.collection,
			item: total.item,
			amount: (total.amount / 2),
		};
		let player = do_all_set_price(supply.clone(), 10 * unit(GAKI));
		assert_eq!(
			ReservedBalanceOf::<Test>::get((player.clone(), supply.collection, supply.item)),
			supply.amount
		);
		assert_ok!(PalletGame::add_retail_supply(
			RuntimeOrigin::signed(player.clone()),
			0,
			supply
		));
		assert_eq!(
			ReservedBalanceOf::<Test>::get((player.clone(), total.collection, total.item)),
			total.amount
		);
	})
}

#[test]
pub fn add_retail_supply_should_failss() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = do_all_set_price(TEST_BUNDLE[0].clone(), 10 * unit(GAKI));
		let player1 = new_account(1, 1000 * unit(GAKI));

		assert_err!(
			PalletGame::add_retail_supply(
				RuntimeOrigin::signed(player1.clone()),
				0,
				TEST_BUNDLE[0].clone()
			),
			Error::<Test>::NoPermission
		);

		assert_err!(
			PalletGame::add_retail_supply(
				RuntimeOrigin::signed(player.clone()),
				0,
				TEST_BUNDLE1[1].clone()
			),
			Error::<Test>::IncorrectCollection
		);

		assert_err!(
			PalletGame::add_retail_supply(
				RuntimeOrigin::signed(player.clone()),
				0,
				TEST_BUNDLE[1].clone()
			),
			Error::<Test>::IncorrectItem
		);

		assert_err!(
			PalletGame::add_retail_supply(
				RuntimeOrigin::signed(player.clone()),
				0,
				TEST_BUNDLE[0].clone()
			),
			Error::<Test>::InsufficientItemBalance
		);
	})
}

#[test]
pub fn set_bundle_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = create_account_with_item(TEST_BUNDLE);

		let price = 100 * unit(GAKI);

		assert_ok!(PalletGame::set_bundle(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE.clone().to_vec(),
			price
		));

		assert_eq!(BundleOf::<Test>::get(0), TEST_BUNDLE.clone().to_vec());
	})
}

#[test]
pub fn set_bundle_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let seller = new_account(0, 10_000 * unit(GAKI));

		for i in 0..(MAX_BUNDLE_VAL + 5) {
			ItemBalanceOf::<Test>::insert((&seller, 0, i), 10);
		}

		{
			let packages: Vec<PackageFor<Test>> = vec![Package::new(0, 0, 11)];
			assert_err!(
				PalletGame::set_bundle(
					RuntimeOrigin::signed(seller.clone()),
					packages.clone(),
					100 * unit(GAKI),
				),
				Error::<Test>::InsufficientItemBalance
			);
		}

		{
			let mut packages: Vec<PackageFor<Test>> = vec![];

			for i in 0..(MAX_BUNDLE_VAL + 5) {
				packages.push(Package::new(0, i, 1));
			}
			assert_err!(
				PalletGame::set_bundle(
					RuntimeOrigin::signed(seller.clone()),
					packages.clone(),
					100 * unit(GAKI),
				),
				Error::<Test>::ExceedMaxBundle
			);
		}
	})
}

#[test]
pub fn buy_bundle_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let price = 10 * unit(GAKI);
		let seller = do_all_set_bundle(TEST_BUNDLE.clone().to_vec(), price);
		let buyer = new_account(1, 10_000 * unit(GAKI));

		let seller_before_balance = Balances::free_balance(&seller);
		let buyer_before_balance = Balances::free_balance(&buyer);

		assert_ok!(PalletGame::buy_bundle(
			RuntimeOrigin::signed(buyer.clone()),
			0,
			price
		));
		for i in 0..TEST_BUNDLE.len() as u32 {
			assert_eq!(ItemBalanceOf::<Test>::get((buyer.clone(), 0, i)), 10);
		}
		for i in 0..TEST_BUNDLE.len() as u32 {
			assert_eq!(ItemBalanceOf::<Test>::get((seller.clone(), 0, i)), 0);
		}
		assert_eq!(
			Balances::free_balance(&seller),
			seller_before_balance + price + BUNDLE_DEPOSIT_VAL
		);
		assert_eq!(Balances::free_balance(&buyer), buyer_before_balance - price);
	})
}

#[test]
pub fn buy_bundle_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let price = 100 * unit(GAKI);
		let seller = do_all_set_bundle(TEST_BUNDLE.clone().to_vec(), price);

		let buyer = new_account(1, 1 * unit(GAKI));

		assert_err!(
			PalletGame::buy_bundle(RuntimeOrigin::signed(buyer.clone()), 0, price * unit(GAKI)),
			TokenError::FundsUnavailable,
		);

		let buyer = new_account(1, 1000 * unit(GAKI));
		assert_err!(
			PalletGame::buy_bundle(
				RuntimeOrigin::signed(buyer.clone()),
				0,
				price - 1 * unit(GAKI)
			),
			Error::<Test>::BidTooLow
		);
	})
}

#[test]
pub fn do_cancel_set_price_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = do_all_set_price(TEST_BUNDLE[0].clone(), unit(GAKI));

		let before_balance = Balances::free_balance(&player);

		assert_ok!(PalletGame::do_cancel_price(&0, &player.clone(),));

		assert_eq!(BundleOf::<Test>::get(0), [].to_vec());
		assert_eq!(TradeConfigOf::<Test>::get(0), None);

		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 10);
		assert_eq!(ReservedBalanceOf::<Test>::get((player.clone(), 0, 0)), 0);
		assert_eq!(
			Balances::free_balance(&player),
			before_balance + BUNDLE_DEPOSIT_VAL
		);
	});
}

#[test]
pub fn do_cancel_set_bundle_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = do_all_set_bundle(TEST_BUNDLE.clone().to_vec(), unit(GAKI));

		let before_balance = Balances::free_balance(&player);
		assert_ok!(PalletGame::do_cancel_bundle(&0, &player.clone(),));

		assert_eq!(BundleOf::<Test>::get(0), [].to_vec());
		assert_eq!(TradeConfigOf::<Test>::get(0), None);

		for i in 0..TEST_BUNDLE.len() as u32 {
			assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, i)), 10);
			assert_eq!(ReservedBalanceOf::<Test>::get((player.clone(), 0, i)), 0);
		}
		assert_eq!(
			Balances::free_balance(&player),
			before_balance + BUNDLE_DEPOSIT_VAL
		);
	});
}

#[test]
pub fn cancel_set_wishlist_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = create_account_with_item(TEST_BUNDLE);
		let price = 3 * unit(GAKI);

		assert_ok!(PalletGame::set_wishlist(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE.clone().to_vec(),
			price,
		));

		let before_balance = Balances::free_balance(&player);
		assert_ok!(PalletGame::do_cancel_wishlist(&0, &player.clone(),));

		assert_eq!(BundleOf::<Test>::get(0), [].to_vec());
		assert_eq!(TradeConfigOf::<Test>::get(0), None);

		assert_eq!(
			Balances::free_balance(&player),
			before_balance + BUNDLE_DEPOSIT_VAL
		);
	});
}

#[test]
pub fn do_cancel_set_swap_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = create_account_with_item(TEST_BUNDLE);

		let price = 100 * unit(GAKI);

		assert_ok!(PalletGame::set_swap(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE.clone().to_vec(),
			TEST_BUNDLE1.clone().to_vec(),
			Some(price)
		));

		let before_balance = Balances::free_balance(&player);
		assert_ok!(PalletGame::do_cancel_swap(&0, &player.clone(),));
		assert_eq!(BundleOf::<Test>::get(0), [].to_vec());
		assert_eq!(TradeConfigOf::<Test>::get(0), None);

		assert_eq!(
			Balances::free_balance(&player),
			before_balance + BUNDLE_DEPOSIT_VAL
		);
	});
}

#[test]
pub fn set_wishlist_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let buyer = new_account(3, 1000 * unit(GAKI));
		let price = 3 * unit(GAKI);
		let before_balance = Balances::free_balance(&buyer);
		assert_ok!(PalletGame::set_wishlist(
			RuntimeOrigin::signed(buyer.clone()),
			TEST_BUNDLE.clone().to_vec(),
			price,
		));

		assert_eq!(BundleOf::<Test>::get(0), TEST_BUNDLE.clone().to_vec());
		assert_eq!(
			TradeConfigOf::<Test>::get(0).unwrap(),
			TradeConfig {
				trade: TradeType::Wishlist,
				owner: buyer.clone(),
				maybe_price: Some(price),
				maybe_required: None,
			}
		);
		assert_eq!(
			Balances::free_balance(&buyer),
			before_balance - (BUNDLE_DEPOSIT_VAL + price)
		);
	})
}

#[test]
pub fn fill_wishlist_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let player = create_account_with_item(TEST_BUNDLE);
		let price = 3 * unit(GAKI);

		let buyer = new_account(3, 1000 * unit(GAKI));
		assert_ok!(PalletGame::set_wishlist(
			RuntimeOrigin::signed(buyer.clone()),
			TEST_BUNDLE.clone().to_vec(),
			price,
		));

		let before_player_balance = Balances::free_balance(&player);
		assert_ok!(PalletGame::fill_wishlist(
			RuntimeOrigin::signed(player.clone()),
			0,
			price
		));

		assert_eq!(BundleOf::<Test>::get(0), [].to_vec());
		assert_eq!(TradeConfigOf::<Test>::get(0), None);

		for i in 0..TEST_BUNDLE.len() as u32 {
			assert_eq!(ItemBalanceOf::<Test>::get((&player, 0, i)), 0);
			assert_eq!(ItemBalanceOf::<Test>::get((&buyer, 0, i)), 10);
		}
		assert_eq!(Balances::reserved_balance(&buyer), 0);
		assert_eq!(
			Balances::free_balance(&player),
			before_player_balance + price
		);
	})
}

#[test]
pub fn set_swap_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = create_account_with_item(TEST_BUNDLE);

		let price = 100 * unit(GAKI);

		let player_balance = Balances::free_balance(&player);
		assert_ok!(PalletGame::set_swap(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE.clone().to_vec(),
			TEST_BUNDLE1.clone().to_vec(),
			Some(price)
		));

		assert_eq!(
			Balances::free_balance(&player),
			player_balance - BUNDLE_DEPOSIT_VAL
		);
		assert_eq!(BundleOf::<Test>::get(0), TEST_BUNDLE.clone().to_vec());

		for i in 0..TEST_BUNDLE.len() as u32 {
			assert_eq!(ItemBalanceOf::<Test>::get((&player, 0, i)), 0);
		}

		assert_eq!(
			TradeConfigOf::<Test>::get(0).unwrap(),
			TradeConfig {
				trade: TradeType::Swap,
				owner: player.clone(),
				maybe_price: Some(price),
				maybe_required: Some(
					BundleFor::<Test>::try_from(TEST_BUNDLE1.clone().to_vec()).unwrap()
				),
			}
		);
	})
}

#[test]
pub fn claim_swap_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player1 = create_account_with_item(TEST_BUNDLE);
		let player2 = create_account_with_item(TEST_BUNDLE1);

		let price = 100 * unit(GAKI);

		assert_ok!(PalletGame::set_swap(
			RuntimeOrigin::signed(player1.clone()),
			TEST_BUNDLE.clone().to_vec(),
			TEST_BUNDLE1.clone().to_vec(),
			Some(price)
		));

		let player1_balance = Balances::free_balance(&player1);
		let player2_balance = Balances::free_balance(&player2);

		assert_ok!(PalletGame::claim_swap(
			RuntimeOrigin::signed(player2.clone()),
			0,
			Some(price)
		));

		assert_eq!(
			Balances::free_balance(&player1),
			player1_balance + price + BUNDLE_DEPOSIT_VAL
		);
		assert_eq!(Balances::free_balance(&player2), player2_balance - price);

		for i in 0..TEST_BUNDLE.len() as u32 {
			assert_eq!(ItemBalanceOf::<Test>::get((&player1, 1, i)), 10);
			assert_eq!(ItemBalanceOf::<Test>::get((&player2, 0, i)), 10);
		}
	})
}

#[test]
pub fn set_auction_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = create_account_with_item(TEST_BUNDLE);

		let price = 100 * unit(GAKI);

		let player_balance = Balances::free_balance(&player);

		assert_ok!(PalletGame::set_auction(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE.clone().to_vec(),
			Some(price),
			1,
			1,
		));
		assert_eq!(
			Balances::free_balance(&player),
			player_balance - BUNDLE_DEPOSIT_VAL
		);
		for i in 0..TEST_BUNDLE.len() as u32 {
			assert_eq!(ItemBalanceOf::<Test>::get((&player, 0, i)), 0);
			assert_eq!(ReservedBalanceOf::<Test>::get((&player, 0, i)), 10);
		}
	})
}

#[test]
pub fn bid_auction_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = create_account_with_item(TEST_BUNDLE);
		assert_ok!(PalletGame::set_auction(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE.clone().to_vec(),
			Some(100 * unit(GAKI)),
			1,
			10,
		));

		let bidder = new_account(1, 1000 * unit(GAKI));
		let bid = 200 * unit(GAKI);
		let bidder_balance = Balances::free_balance(&bidder);
		assert_ok!(PalletGame::bid_auction(
			RuntimeOrigin::signed(bidder.clone()),
			0,
			bid
		));
		assert_eq!(Balances::free_balance(&bidder), bidder_balance - bid);

		run_to_block(2);

		assert_ok!(PalletGame::bid_auction(
			RuntimeOrigin::signed(bidder.clone()),
			0,
			bid * 2
		));

		assert_eq!(Balances::free_balance(&bidder), bidder_balance - (bid * 2));
	})
}

#[test]
pub fn bid_auction_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = create_account_with_item(TEST_BUNDLE);
		assert_ok!(PalletGame::set_auction(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE.clone().to_vec(),
			Some(100 * unit(GAKI)),
			2,
			10,
		));

		let bidder = new_account(1, 1000 * unit(GAKI));
		assert_err!(
			PalletGame::bid_auction(RuntimeOrigin::signed(bidder.clone()), 0, 50,),
			Error::<Test>::AuctionNotStarted
		);
		run_to_block(2);
		assert_err!(
			PalletGame::bid_auction(RuntimeOrigin::signed(bidder.clone()), 0, 50,),
			Error::<Test>::BidTooLow
		);

		run_to_block(11);
		assert_ok!(PalletGame::bid_auction(
			RuntimeOrigin::signed(bidder.clone()),
			0,
			200 * unit(GAKI)
		));

		let bidder1 = new_account(2, 1000 * unit(GAKI));
		assert_err!(
			PalletGame::bid_auction(RuntimeOrigin::signed(bidder1.clone()), 0, 200 * unit(GAKI)),
			Error::<Test>::BidTooLow
		);

		run_to_block(12);
		assert_err!(
			PalletGame::bid_auction(RuntimeOrigin::signed(bidder1.clone()), 0, 300 * unit(GAKI)),
			Error::<Test>::AuctionEnded
		);
	})
}

#[test]
pub fn claim_auction_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = create_account_with_item(TEST_BUNDLE);
		assert_ok!(PalletGame::set_auction(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE.clone().to_vec(),
			Some(100 * unit(GAKI)),
			1,
			10,
		));

		run_to_block(2);
		let bids = [
			(new_account(1, 1000 * unit(GAKI)), 100 * unit(GAKI)),
			(new_account(2, 1000 * unit(GAKI)), 200 * unit(GAKI)),
			(new_account(3, 1000 * unit(GAKI)), 400 * unit(GAKI)),
			(new_account(4, 1000 * unit(GAKI)), 500 * unit(GAKI)),
		];

		for i in 0..bids.len() {
			run_to_block(2 + i as u64);
			assert_ok!(PalletGame::bid_auction(
				RuntimeOrigin::signed(bids[i].0.clone()),
				0,
				bids[i].1,
			));
		}

		let player_balance = Balances::free_balance(&player);
		run_to_block(11);
		assert_ok!(PalletGame::claim_auction(
			RuntimeOrigin::signed(player.clone()),
			0
		));

		let winner = bids[3].clone();

		for i in 0..TEST_BUNDLE.len() as u32 {
			assert_eq!(ItemBalanceOf::<Test>::get((&winner.0.clone(), 0, i)), 10);
		}

		assert_eq!(
			Balances::free_balance(&player),
			player_balance + winner.1 + BUNDLE_DEPOSIT_VAL
		);

		for i in 0..(bids.len() - 1) {
			assert_eq!(
				Balances::free_balance(&bids[i].0.clone()),
				1000 * unit(GAKI)
			);
		}
	})
}

#[test]
pub fn claim_auction_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = create_account_with_item(TEST_BUNDLE);

		assert_err!(
			PalletGame::claim_auction(RuntimeOrigin::signed(player.clone()), 0),
			Error::<Test>::UnknownAuction
		);

		assert_ok!(PalletGame::set_auction(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE.clone().to_vec(),
			Some(100 * unit(GAKI)),
			1,
			10,
		));

		assert_err!(
			PalletGame::claim_auction(RuntimeOrigin::signed(player.clone()), 0),
			Error::<Test>::AuctionInProgress
		);
	})
}

#[test]
pub fn set_buy_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let player = new_account(0, 1000 * unit(GAKI));
		let player_balance = Balances::free_balance(&player);

		let unit_price = 5 * unit(GAKI);
		assert_ok!(PalletGame::set_buy(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE[0].clone(),
			unit_price
		));

		assert_eq!(
			Balances::free_balance(&player),
			player_balance -
				BUNDLE_DEPOSIT_VAL -
				(unit_price * TEST_BUNDLE[0].clone().amount as u128)
		);
	})
}

#[test]
pub fn claim_set_buy_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let player = new_account(0, 1000 * unit(GAKI));

		let unit_price = 5 * unit(GAKI);
		assert_ok!(PalletGame::set_buy(
			RuntimeOrigin::signed(player.clone()),
			TEST_BUNDLE[0].clone(),
			unit_price
		));

		let seller = create_account_with_item(TEST_BUNDLE);
		let amount = TEST_BUNDLE[0].clone().amount / 2;
		let seller_balance = Balances::free_balance(&seller);

		assert_ok!(PalletGame::claim_set_buy(
			RuntimeOrigin::signed(seller.clone()),
			0,
			amount,
			unit_price
		));

		assert_eq!(
			Balances::free_balance(&seller),
			seller_balance + (unit_price * amount as u128)
		);
		assert_eq!(
			ItemBalanceOf::<Test>::get((
				player.clone(),
				TEST_BUNDLE[0].clone().collection,
				TEST_BUNDLE[0].clone().item
			)),
			amount
		);

		assert_eq!(
			ItemBalanceOf::<Test>::get((
				seller.clone(),
				TEST_BUNDLE[0].clone().collection,
				TEST_BUNDLE[0].clone().item
			)),
			amount
		);
	})
}

#[test]
fn create_stable_pool_should_works() {
	new_test_ext().execute_with(|| {
		let (owner, admin) = do_create_game();
		do_create_collection(0, &admin);

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0,
			default_item_config(),
			None
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			1,
			default_item_config(),
			None
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			2,
			default_item_config(),
			None
		));

		let owner_balance = Balances::free_balance(owner.clone());
		assert_ok!(PalletGame::create_stable_pool(
			RuntimeOrigin::signed(owner.clone()),
			TEST_TABLE.clone().to_vec(),
			100,
			admin.clone()
		));

		assert_eq!(
			Balances::free_balance(owner.clone()),
			owner_balance - MINING_DEPOSIT_VAL
		);
		assert_eq!(LootTableOf::<Test>::get(0), TEST_TABLE.to_vec());
	})
}

fn create_stable_pool_should_fails() {
	new_test_ext().execute_with(|| {
		let (owner, admin) = do_create_game();
		do_create_collection(0, &admin);

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0,
			default_item_config(),
			None
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			1,
			default_item_config(),
			None
		));

		assert_err!(
			PalletGame::create_stable_pool(
				RuntimeOrigin::signed(admin.clone()),
				TEST_TABLE.clone()[0..1].to_vec(),
				100,
				admin.clone()
			),
			Error::<Test>::NoPermission
		);

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			2,
			default_item_config(),
			Some(1000)
		));

		assert_err!(
			PalletGame::create_stable_pool(
				RuntimeOrigin::signed(owner.clone()),
				TEST_TABLE.clone().to_vec(),
				100,
				admin.clone()
			),
			Error::<Test>::NotInfiniteSupply
		);
	})
}

// #[test]
// fn mint_stable_pool_should_works() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);
// 		let mint_fee = 2 * unit(GAKI);
// 		let (owner, _) = do_all_create_stable_pool(mint_fee, TEST_TABLE.clone().to_vec());
// 		let player = new_account(2, 1000_000 * unit(GAKI));
// 		{
// 			let owner_balance = Balances::free_balance(owner.clone());
// 			let player_balance = Balances::free_balance(player.clone());

// 			let amount = 10;
// 			assert_ok!(PalletGame::mint(
// 				RuntimeOrigin::signed(player.clone()),
// 				0,
// 				player.clone(),
// 				amount,
// 			));

// 			assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 10);
// 			assert_eq!(
// 				Balances::free_balance(player.clone()),
// 				player_balance - (mint_fee * amount as u128 as u128)
// 			);
// 			assert_eq!(
// 				Balances::free_balance(owner.clone()),
// 				owner_balance + (mint_fee * amount as u128 as u128)
// 			);
// 		}
// 	})
// }
