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
	game::{Bundle, Package},
};
use pallet_nfts::{
	CollectionRole, CollectionRoles, CollectionSettings, ItemSettings, MintSettings, MintType,
};
use sp_core::bounded::BoundedVec;

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

fn create_items(
	who: &sr25519::Public,
	collection_config: &CollectionConfigFor<Test>,
	item_config: &ItemConfig,
	item_id: u32,
	amount: u32,
) {
	assert_ok!(PalletGame::create_game(
		RuntimeOrigin::signed(who.clone()),
		who.clone()
	));

	assert_ok!(PalletGame::create_game_colletion(
		RuntimeOrigin::signed(who.clone()),
		0,
		who.clone(),
		*collection_config,
	));

	assert_ok!(PalletGame::create_item(
		RuntimeOrigin::signed(who.clone()),
		0,
		item_id,
		*item_config,
		amount
	));
}

#[test]
fn create_first_game_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let owner = new_account(0, before_balance);

		let admin = new_account(1, 3 * unit(GAKI));

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
fn set_swap_fee_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let fee = Percent::from_parts(30);
		let start_block = 100;

		let owner = new_account(0, before_balance);
		let admin = new_account(1, 3 * unit(GAKI));
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

		let owner = new_account(0, before_balance);
		let admin = new_account(1, 3 * unit(GAKI));
		let not_admin = new_account(2, 3 * unit(GAKI));

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
		let owner = new_account(0, before_balance);

		let admin = new_account(1, 3 * unit(GAKI));

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

		assert_eq!(CollectionsOf::<Test>::get(0)[0], 0);
		assert_eq!(GameOf::<Test>::get(0), Some(0));

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(admin.clone()),
			0,
			admin.clone(),
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
			let before_balance = 3 * unit(GAKI);
			let owner = new_account(0, before_balance);
			let admin = new_account(1, 3 * unit(GAKI));
			let acc = new_account(3, 3 * unit(GAKI));

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
		let owner = new_account(0, before_balance);

		let admin = new_account(1, 3 * unit(GAKI));

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
		let owner = new_account(0, before_balance);

		let admin = new_account(1, 3 * unit(GAKI));

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

		assert_eq!(CollectionsOf::<Test>::get(0), [0, 1].to_vec());
		assert_eq!(GameOf::<Test>::get(0).unwrap(), 0);
		assert_eq!(GameOf::<Test>::get(1).unwrap(), 0);
	})
}

#[test]
fn create_item_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let before_balance = 3 * unit(GAKI);
		let owner = new_account(0, before_balance);

		let admin = new_account(1, 3 * unit(GAKI));

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
		let owner = new_account(0, before_balance);

		let admin = new_account(1, 3 * unit(GAKI));

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
		let owner = new_account(0, before_balance);

		let admin = new_account(1, 3 * unit(GAKI));
		let mint_fee: u128 = 3 * unit(GAKI);

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			admin.clone(),
		));

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(admin.clone()),
			0,
			admin.clone(),
			collection_config(mint_fee),
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
		let player = new_account(2, before_balance);
		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			3
		));
		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 1);
		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 1)), 1);
		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 2)), 1);
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
		let owner = new_account(0, 3 * unit(GAKI));

		let admin = new_account(1, 3 * unit(GAKI));
		let mint_fee: u128 = 3 * unit(GAKI);

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			admin.clone(),
		));

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(admin.clone()),
			0,
			admin.clone(),
			collection_config(mint_fee),
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(admin.clone()),
			0,
			0,
			default_item_config(),
			MAX_ITEM_MINT_VAL + 1
		));

		let player = new_account(2, 3000 * unit(GAKI));
		assert_err!(
			PalletGame::mint(
				RuntimeOrigin::signed(player.clone()),
				0,
				player.clone(),
				MAX_ITEM_MINT_VAL + 1
			),
			Error::<Test>::ExceedAllowedAmount
		);

		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			MAX_ITEM_MINT_VAL,
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
		let owner = new_account(0, 3 * unit(GAKI));

		assert_ok!(PalletGame::create_game(
			RuntimeOrigin::signed(owner.clone()),
			owner.clone()
		));

		assert_ok!(PalletGame::create_game_colletion(
			RuntimeOrigin::signed(owner.clone()),
			0,
			owner.clone(),
			collection_config(0),
		));

		assert_ok!(PalletGame::create_item(
			RuntimeOrigin::signed(owner.clone()),
			0,
			0,
			default_item_config(),
			100
		));

		let player = new_account(2, 3000 * unit(GAKI));
		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			10
		));
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

		let owner = new_account(0, 10_000 * unit(GAKI));

		create_items(
			&owner,
			&default_collection_config(),
			&default_item_config(),
			0,
			100,
		);

		let player = new_account(2, 3000 * unit(GAKI));
		let dest = new_account(3, 3000 * unit(GAKI));
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

		let owner = new_account(0, 10_000 * unit(GAKI));

		create_items(
			&owner,
			&default_collection_config(),
			&default_item_config(),
			0,
			100,
		);

		let byte = 50;

		let input: UpgradeItemConfig<u32, u128> = UpgradeItemConfig {
			item: 100,
			fee: 3 * unit(GAKI),
		};

		let before_balance = Balances::free_balance(&owner);

		assert_ok!(PalletGame::set_upgrade_item(
			RuntimeOrigin::signed(owner.clone()),
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

#[test]
pub fn upgrade_item_shoud_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let owner = new_account(0, 10_000 * unit(GAKI));

		create_items(
			&owner,
			&default_collection_config(),
			&default_item_config(),
			0,
			100,
		);

		let byte = 50;

		let input: UpgradeItemConfig<u32, u128> = UpgradeItemConfig {
			item: 100,
			fee: 3 * unit(GAKI),
		};

		assert_ok!(PalletGame::set_upgrade_item(
			RuntimeOrigin::signed(owner.clone()),
			0,
			0,
			input.item,
			default_item_config(),
			bvec![0u8; byte],
			1,
			input.fee,
		));

		let player = new_account(3, 10000 * unit(GAKI));

		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			10
		));

		let player_before_balance = Balances::free_balance(&player);
		let owner_before_balance = Balances::free_balance(&owner);

		assert_ok!(PalletGame::upgrade_item(
			RuntimeOrigin::signed(player.clone()),
			0,
			0,
			3
		));

		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 0)), 7);
		assert_eq!(ItemBalanceOf::<Test>::get((player.clone(), 0, 100)), 3);
		assert_eq!(
			Balances::free_balance(&player),
			player_before_balance - (input.fee * 3)
		);
		assert_eq!(
			Balances::free_balance(&owner),
			owner_before_balance + (input.fee * 3)
		);
	})
}

#[test]
pub fn set_price_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let owner = new_account(0, 10_000 * unit(GAKI));

		create_items(
			&owner,
			&default_collection_config(),
			&default_item_config(),
			0,
			100,
		);

		let player = new_account(3, 10000 * unit(GAKI));

		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			10
		));

		let trade_config = TradeConfig {
			price: 3 * unit(GAKI),
			amount: 9,
			min_order_quantity: Some(3),
		};
		let before_balance = Balances::free_balance(&player);

		assert_ok!(PalletGame::set_price(
			RuntimeOrigin::signed(player.clone()),
			0,
			0,
			trade_config.clone()
		));
		assert_eq!(
			Balances::free_balance(&player),
			before_balance - SALE_DEPOSIT_VAL
		);
		assert_eq!(
			TradeConfigOf::<Test>::get((player.clone(), 0, 0)).unwrap(),
			trade_config
		);
	})
}

#[test]
pub fn set_price_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let owner = new_account(0, 10_000 * unit(GAKI));

		let mut item_config = ItemConfig::default();
		item_config.disable_setting(pallet_nfts::ItemSetting::Transferable);

		create_items(&owner, &default_collection_config(), &item_config, 0, 100);

		let player = new_account(3, 10000 * unit(GAKI));

		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(player.clone()),
			0,
			player.clone(),
			10
		));

		let mut trade_config = TradeConfig {
			price: 3 * unit(GAKI),
			amount: 11,
			min_order_quantity: Some(3),
		};

		assert_err!(
			PalletGame::set_price(
				RuntimeOrigin::signed(player.clone()),
				0,
				0,
				trade_config.clone()
			),
			Error::<Test>::InsufficientItemBalance
		);

		trade_config.amount = 1;
		assert_err!(
			PalletGame::set_price(
				RuntimeOrigin::signed(player.clone()),
				0,
				0,
				trade_config.clone()
			),
			Error::<Test>::ItemLocked
		);
	})
}

#[test]
pub fn buy_item_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let owner = new_account(0, 10_000 * unit(GAKI));

		create_items(
			&owner,
			&default_collection_config(),
			&default_item_config(),
			0,
			100,
		);

		let seller = new_account(3, 10000 * unit(GAKI));

		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(seller.clone()),
			0,
			seller.clone(),
			10
		));

		let price = 3 * unit(GAKI);
		let trade_config = TradeConfig {
			price,
			amount: 8,
			min_order_quantity: Some(3),
		};

		assert_ok!(PalletGame::set_price(
			RuntimeOrigin::signed(seller.clone()),
			0,
			0,
			trade_config.clone()
		));

		let buyer = new_account(4, 10000 * unit(GAKI));

		let seller_before_balance = Balances::free_balance(&seller);
		let buyer_before_balance = Balances::free_balance(&buyer);

		assert_ok!(PalletGame::buy_item(
			RuntimeOrigin::signed(buyer.clone()),
			0,
			0,
			seller.clone(),
			3,
			price,
		));

		assert_eq!(ItemBalanceOf::<Test>::get((&seller, 0, 0)), 2);
		assert_eq!(LockBalanceOf::<Test>::get((&seller, 0, 0)), 5);
		assert_eq!(ItemBalanceOf::<Test>::get((&buyer, 0, 0)), 3);

		assert_eq!(
			Balances::free_balance(&seller),
			seller_before_balance + (price * 3)
		);
		assert_eq!(
			Balances::free_balance(&buyer),
			buyer_before_balance - (price * 3)
		);
	})
}

#[test]
pub fn buy_item_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let owner = new_account(0, 10_000 * unit(GAKI));

		create_items(
			&owner,
			&default_collection_config(),
			&default_item_config(),
			0,
			100,
		);

		let seller = new_account(3, 10000 * unit(GAKI));

		assert_ok!(PalletGame::mint(
			RuntimeOrigin::signed(seller.clone()),
			0,
			seller.clone(),
			10
		));

		let price = 3 * unit(GAKI);
		let trade_config = TradeConfig {
			price,
			amount: 10,
			min_order_quantity: Some(3),
		};

		assert_ok!(PalletGame::set_price(
			RuntimeOrigin::signed(seller.clone()),
			0,
			0,
			trade_config.clone()
		));

		let buyer = new_account(4, 10000 * unit(GAKI));

		assert_err!(
			PalletGame::buy_item(
				RuntimeOrigin::signed(buyer.clone()),
				0,
				0,
				seller.clone(),
				2,
				price,
			),
			Error::<Test>::AmountUnacceptable
		);

		assert_err!(
			PalletGame::buy_item(
				RuntimeOrigin::signed(buyer.clone()),
				0,
				0,
				seller.clone(),
				4,
				price - (1 * unit(GAKI)),
			),
			Error::<Test>::BidTooLow
		);

		assert_ok!(PalletGame::buy_item(
			RuntimeOrigin::signed(buyer.clone()),
			0,
			0,
			seller.clone(),
			8,
			price,
		));

		assert_err!(
			PalletGame::buy_item(
				RuntimeOrigin::signed(buyer.clone()),
				0,
				0,
				seller.clone(),
				4,
				price,
			),
			Error::<Test>::BuyAllOnly
		);

		assert_ok!(PalletGame::buy_item(
			RuntimeOrigin::signed(buyer.clone()),
			0,
			0,
			seller.clone(),
			2,
			price,
		));

		assert_err!(
			PalletGame::buy_item(
				RuntimeOrigin::signed(buyer.clone()),
				0,
				0,
				seller.clone(),
				4,
				price,
			),
			Error::<Test>::SoldOut
		);
	})
}

#[test]
pub fn set_bundle_should_works() {
	new_test_ext().execute_with(|| {
		let count = 2_u32;
		let seller = new_account(0, 10_000 * unit(GAKI));

		for i in 0..count {
			ItemBalanceOf::<Test>::insert((&seller, 0, i), 10);
		}

		let mut packages: Vec<PackageFor<Test>> = vec![];
		for i in 0..count {
			packages.push(Package::new(0, i, 10));
		}

		let price = 100 * unit(GAKI);

		assert_ok!(PalletGame::set_bundle(
			RuntimeOrigin::signed(seller.clone()),
			packages.clone(),
			price
		));

		let bundle_id = PalletGame::get_bundle_id(&packages, &seller);

		assert_eq!(BundleOf::<Test>::get(bundle_id), packages);
	})
}

#[test]
pub fn set_bundle_should_fails() {
	new_test_ext().execute_with(|| {
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
			let packages: Vec<PackageFor<Test>> = vec![Package::new(0, 0, 1)];
			assert_ok!(PalletGame::set_bundle(
				RuntimeOrigin::signed(seller.clone()),
				packages.clone(),
				100 * unit(GAKI),
			));

			assert_err!(
				PalletGame::set_bundle(
					RuntimeOrigin::signed(seller.clone()),
					packages.clone(),
					100 * unit(GAKI),
				),
				Error::<Test>::IdExists
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
	new_test_ext().execute_with(|| {})
}

#[test]
pub fn buy_bundle_should_fails() {
	new_test_ext().execute_with(|| {})
}
