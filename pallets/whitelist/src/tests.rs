use crate::{mock::*, Error, Whitelist, WhitelistSource};
use codec::{Decode, Encode};
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::{
	constant::ID,
	currency::{unit, NativeToken::GAKI},
};
use rustc_hex::ToHex;
use sp_core::{offchain::OffchainWorkerExt, sr25519, H160};
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use sp_runtime::{
	offchain::{testing, TransactionPoolExt},
	Permill,
};
use funding_pool::{PoolOwned, Pools};
use std::{str::FromStr, sync::Arc};
use sp_runtime::offchain::{http, Duration, Timestamp};

#[cfg(feature = "runtime-benchmarks")]
use funding_pool::CustomPool;

fn make_deposit(account: &sr25519::Public, balance: u128) {
	let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(account: u32, balance: u128) -> sr25519::Public {
	let keystore = KeyStore::new();
	let acc: sr25519::Public = keystore
		.sr25519_generate_new(sp_runtime::KeyTypeId::from(account), None)
		.unwrap();
	make_deposit(&acc, balance);
	assert_eq!(Balances::free_balance(&acc), balance);
	return acc
}

fn test_pub() -> sp_core::sr25519::Public {
	sp_core::sr25519::Public::from_raw([1u8; 32])
}

struct VerifyResult {
	result: bool
}

const TEST_URL: &str = "http://whitelist.gafi.network/verify";

fn create_pool(
	account: sr25519::Public,
	account_balance: u128,
	targets: Vec<H160>,
	pool_value: u128,
	tx_limit: u32,
	discount: Permill,
) -> ID {
	assert_ok!(Funding::create_pool(
		RuntimeOrigin::signed(account.clone()),
		targets,
		pool_value,
		discount,
		tx_limit
	));

	assert_eq!(
		Balances::free_balance(&account),
		account_balance - pool_value
	);
	let pool_owned = PoolOwned::<Test>::get(account.clone());
	assert_eq!(pool_owned.len(), 1);
	let new_pool = Pools::<Test>::get(pool_owned[0]).unwrap();
	assert_eq!(new_pool.owner, account);
	assert_eq!(new_pool.tx_limit, tx_limit);
	assert_eq!(new_pool.discount, discount);
	new_pool.id
}

#[test]
fn should_submit_raw_unsigned_transaction_on_chain() {
	let (offchain, offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();

	let keystore = KeyStore::new();

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt(Arc::new(keystore)));

	let player =
		sr25519::Public::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
	let pool_id = [0_u8; 32];

	let pool_id_hex: String = pool_id.to_hex();

	let uri = format!(
		"${:?}?_id=${}&address=${}",
		TEST_URL, pool_id_hex, player
	);
	whitelist_response_work(&mut offchain_state.write(), &uri);

	t.execute_with(|| {
		assert_ok!(PalletWhitelist::verify_and_approve(&uri, player, pool_id));

		// then
		let tx = pool_state.write().transactions.pop().unwrap();
		assert!(pool_state.read().transactions.is_empty());
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		assert_eq!(
			tx.call,
			RuntimeCall::PalletWhitelist(crate::Call::approve_whitelist_unsigned { player, pool_id })
		);
	});
}

#[test]
fn query_whitelist_should_fails() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let pool_id = [0; 32];
		let account_balance = 1_000_000 * unit(GAKI);
		let account = new_account(1, account_balance);

		let player = new_account(2, account_balance);

		assert_err!(
			PalletWhitelist::apply_whitelist(RuntimeOrigin::signed(player.clone()), pool_id),
			Error::<Test>::PoolNotFound.as_str(),
		);

		create_pool(
			account.clone(),
			account_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
			1_000 * unit(GAKI),
			10,
			Permill::from_percent(70),
		);

		let pool_id: ID = *PoolOwned::<Test>::get(account.clone()).last().unwrap();
		assert_err!(
			PalletWhitelist::apply_whitelist(RuntimeOrigin::signed(player.clone()), pool_id),
			Error::<Test>::PoolNotWhitelist.as_str()
		);

		let url = b"http://whitelist.gafi.network/whitelist/verify";

		assert_ok!(PalletWhitelist::enable_whitelist(
			RuntimeOrigin::signed(account.clone()),
			pool_id,
			url.to_vec()
		));

		assert_ok!(PalletWhitelist::apply_whitelist(
			RuntimeOrigin::signed(player.clone()),
			pool_id
		));

		assert_err!(
			PalletWhitelist::apply_whitelist(RuntimeOrigin::signed(player.clone()), pool_id),
			Error::<Test>::AlreadyWhitelist.as_str()
		);

		assert_ok!(PalletWhitelist::approve_whitelist(RuntimeOrigin::signed(account.clone()), player.clone(), pool_id));

		assert_err!(
			PalletWhitelist::apply_whitelist(RuntimeOrigin::signed(player.clone()), pool_id),
			Error::<Test>::AlreadyJoined.as_str()
		);
	})
}

#[test]
fn should_make_http_call_and_parse_result() {
	let (offchain, state) = testing::TestOffchainExt::new();
	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));

	// let url = format!("?id=d63de0e8c06ceacebd5bbb54500d82d061fb92f3a7ec1250dfefd99ec6de2456&address=d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
	whitelist_response_work(&mut state.write(), &TEST_URL);

	t.execute_with(|| {
		// when
		let verify = PalletWhitelist::fetch_whitelist(&TEST_URL).unwrap();
		// then
		assert_eq!(verify, true);
	});
}

fn whitelist_response_work(state: &mut testing::OffchainState, uri: &str) {
	state.expect_request(testing::PendingRequest {
		method: "GET".into(),
		uri: uri.into(),
		response: Some(br#"{"result": true}"#.to_vec()),
		sent: true,
		..Default::default()
	});
}

#[test]
fn make_http_call_and_parse_result_should_fail() {
	let (offchain, state) = testing::TestOffchainExt::new();
	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));

	whitelist_response_fail(&mut state.write(), &TEST_URL);

	t.execute_with(|| {
		// when
		let verify = PalletWhitelist::fetch_whitelist(&TEST_URL).unwrap();
		// then
		assert_eq!(verify, false);
	});
}

fn whitelist_response_fail(state: &mut testing::OffchainState, uri: &str) {
	state.expect_request(testing::PendingRequest {
		method: "GET".into(),
		uri: uri.into(),
		response: Some(br#"{"result": false}"#.to_vec()),
		sent: true,
		..Default::default()
	});
}

#[test]
fn get_uri_should_works() {
	let pool_id = [
		207, 230, 173, 83, 217, 51, 140, 68, 153, 115, 233, 133, 14, 172, 41, 148, 140, 128, 69,
		225, 252, 179, 101, 159, 177, 181, 25, 72, 221, 222, 133, 111,
	];

	let player =
		sr25519::Public::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();

	let link = "http://whitelist.gafi.network/verify";

	let uri = PalletWhitelist::get_api(link, pool_id, &player);

	assert_eq!(uri, "http://whitelist.gafi.network/verify?id=cfe6ad53d9338c449973e9850eac29948c8045e1fcb3659fb1b51948ddde856f&address=d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
}

#[test]
fn enable_whitelist_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
		let account = new_account(0, account_balance);
		let pool_value = 1000 * unit(GAKI);
		create_pool(
			account.clone(),
			account_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
			pool_value,
			10,
			Permill::from_percent(70),
		);

		let pool_id: ID = *PoolOwned::<Test>::get(account.clone()).last().unwrap();

		let url = b"http://whitelist.gafi.network/whitelist/verify";

		assert_ok!(PalletWhitelist::enable_whitelist(
			RuntimeOrigin::signed(account.clone()),
			pool_id,
			url.to_vec()
		));

		assert_eq!(Balances::reserved_balance(account.clone()), WHITELIST_FEE);

		assert_eq!(WhitelistSource::<Test>::get(pool_id).unwrap().0, url.to_vec());

		assert_ok!(PalletWhitelist::enable_whitelist(
			RuntimeOrigin::signed(account),
			pool_id,
			b"".to_vec(),
		));

		assert_eq!(Balances::reserved_balance(account.clone()), WHITELIST_FEE);

		assert_eq!(
			WhitelistSource::<Test>::get(pool_id).unwrap().0.to_vec(),
			b"".to_vec()
		);
	})
}

#[test]
fn enable_whitelist_fails() {
	new_test_ext().execute_with(|| {
        run_to_block(1);
        let account_balance = 1_000_000 * unit(GAKI);
        let account = new_account(0, account_balance);
        let account2 = new_account(1, account_balance);
        let pool_value = 1000 * unit(GAKI);
        create_pool(
            account.clone(),
            account_balance,
            vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
            pool_value,
            10,
            Permill::from_percent(70),
        );

        let pool_id: ID = *PoolOwned::<Test>::get(account.clone()).last().unwrap();

        let url = b"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

        assert_err!(PalletWhitelist::enable_whitelist(RuntimeOrigin::signed(account.clone()), pool_id, url.to_vec()), Error::<Test>::URLTooLong);
        assert_err!(PalletWhitelist::enable_whitelist(RuntimeOrigin::signed(account.clone()), [0_u8; 32], b"".to_vec()), Error::<Test>::PoolNotFound);
        assert_err!(PalletWhitelist::enable_whitelist(RuntimeOrigin::signed(account2.clone()), pool_id, b"".to_vec()), Error::<Test>::NotPoolOwner);
    })
}

#[test]
fn get_url_work() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
		let account = new_account(0, account_balance);
		let pool_value = 1000 * unit(GAKI);
		create_pool(
			account.clone(),
			account_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
			pool_value,
			10,
			Permill::from_percent(70),
		);

		let pool_id: ID = *PoolOwned::<Test>::get(account.clone()).last().unwrap();

		let url = b"http://whitelist.gafi.network/whitelist/verify";

		assert_ok!(PalletWhitelist::enable_whitelist(
			RuntimeOrigin::signed(account.clone()),
			pool_id,
			url.to_vec()
		));

		let url_str = PalletWhitelist::get_url(pool_id);

		println!("URL: {:?}", url_str.clone().unwrap());

		assert_eq!(url_str.unwrap().as_bytes(), url);
	})
}

#[test]
fn withdraw_whitelist_works() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let account_balance = 1_000_000 * unit(GAKI);
		let account = new_account(0, account_balance);
		let pool_value = 1000 * unit(GAKI);
		create_pool(
			account.clone(),
			account_balance,
			vec![H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap()],
			pool_value,
			10,
			Permill::from_percent(70),
		);

		let pool_id: ID = *PoolOwned::<Test>::get(account.clone()).last().unwrap();

		let url = b"http://whitelist.gafi.network/whitelist/verify";

		assert_ok!(PalletWhitelist::enable_whitelist(
			RuntimeOrigin::signed(account.clone()),
			pool_id,
			url.to_vec()
		));

		assert_eq!(Balances::reserved_balance(account.clone()), WHITELIST_FEE);

		assert_ok!(PalletWhitelist::withdraw_whitelist(
			RuntimeOrigin::signed(account.clone()),
			pool_id
		));

		assert_eq!(WhitelistSource::<Test>::get(pool_id), None);

		assert_eq!(Balances::reserved_balance(account.clone()), 0);
	})
}
