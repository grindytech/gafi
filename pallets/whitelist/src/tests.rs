use crate::{mock::*, Error, Whitelist};
use codec::{Decode, Encode};
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
};
use rustc_hex::{ToHex};
use sp_core::{
	offchain::{OffchainWorkerExt},
	sr25519,
};
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use sp_runtime::{
	offchain::{testing, TransactionPoolExt},
};
use std::{str::FromStr, sync::Arc};

#[cfg(feature = "runtime-benchmarks")]
use sponsored_pool::CustomPool;

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
		"http://whitelist.gafi.network/whitelist/verify?pool_id=${}&address=${}",
		pool_id_hex, player
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
			Call::PalletWhitelist(crate::Call::approve_whitelist_unsigned { player, pool_id })
		);
	});
}

#[test]
fn query_whitelist_should_fail_pool_not_found() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(1);
		let pool_id = [0; 32];
		let account_balance = 1_000_000 * unit(GAKI);

		let player = new_account(1, account_balance);

		assert_err!(
			PalletWhitelist::query_whitelist(Origin::signed(player.clone()), pool_id),
			Error::<Test>::PoolNotFound
		);
	})
}

#[test]
fn should_make_http_call_and_parse_result() {
	let (offchain, state) = testing::TestOffchainExt::new();
	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));

	let pool_id = "0x3a77d059474c1143d0d9cfc55f1d8601099a37c30c943f2807d6a7aa9eddd386";

	let alice =
		sr25519::Public::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();

	let uri = format!(
		"http://whitelist.gafi.network/whitelist/verify?pool_id={}&address={:#}",
		pool_id, alice
	);

	whitelist_response_work(&mut state.write(), &uri);

	// let id = match str::from_utf8(&pool_id) {
	// 	Ok(v) => v,
	// 	Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
	// };

	t.execute_with(|| {
		// when
		let verify = PalletWhitelist::fetch_whitelist(&uri).unwrap();
		// then
		assert_eq!(verify, true);
	});
}

fn whitelist_response_work(state: &mut testing::OffchainState, uri: &str) {
	state.expect_request(testing::PendingRequest {
		method: "GET".into(),
		uri: uri.into(),
		response: Some(br#"true"#.to_vec()),
		sent: true,
		..Default::default()
	});
}

#[test]
fn make_http_call_and_parse_result_should_fail() {
	let (offchain, state) = testing::TestOffchainExt::new();
	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));

	let pool_id = "0x3a77d059474c1143d0d9cfc55f1d8601099a37c30c943f2807d6a7aa9eddd386";

	let dave =
		sr25519::Public::from_str("5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy").unwrap();

	let uri = format!(
		"http://whitelist.gafi.network/whitelist/verify?pool_id={}&address={:#}",
		pool_id, dave
	);
	whitelist_response_fail(&mut state.write(), &uri);

	t.execute_with(|| {
		// when
		let verify = PalletWhitelist::fetch_whitelist(&uri).unwrap();
		// then
		assert_eq!(verify, false);
	});
}

fn whitelist_response_fail(state: &mut testing::OffchainState, uri: &str) {
	state.expect_request(testing::PendingRequest {
		method: "GET".into(),
		uri: uri.into(),
		response: Some(br#"false"#.to_vec()),
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

	let player = sr25519::Public::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();

	let link = "http://whitelist.gafi.network/whitelist/verify";

	let uri = PalletWhitelist::get_uri(link, pool_id, &player);

	assert_eq!(uri, "http://whitelist.gafi.network/whitelist/verify?pool_id=cfe6ad53d9338c449973e9850eac29948c8045e1fcb3659fb1b51948ddde856f&address=d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");

}
