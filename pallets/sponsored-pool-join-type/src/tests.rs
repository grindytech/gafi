use super::*;
use crate::mock::*;

use frame_support::{assert_noop, assert_ok, traits::Currency, assert_err};
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	pool::{SponsoredPoolJoinType, SponsoredPoolJoinTypeHandle},
};
use sp_runtime::{AccountId32, DispatchError};

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
fn join_type_should_work() {
	ExtBuilder::default().build_and_execute(|| {
		let account_balance = 1_000_000 * unit(GAKI);
		run_to_block(1);
		let account = new_account([0_u8; 32], account_balance);
		let url: Vec<u8> = Vec::<u8>::from("https://google.com");
		let long_url: Vec<u8> = Vec::<u8>::from([100; 260]);
		let pool_id = [10; 32];
		let join = SponsoredPoolJoin::set_join_type(
			pool_id,
			SponsoredPoolJoinType::Default,
			url.clone(),
			account.clone(),
		);
		assert_eq!(join, Ok(()));
		assert_eq!(
			SponsoredPoolJoin::get_join_type(pool_id),
			Some((SponsoredPoolJoinType::Default, url.clone()))
		);
		let reset = SponsoredPoolJoin::reset(pool_id, account.clone());
		assert_eq!(reset, Ok(()));

		let join = SponsoredPoolJoin::set_join_type(
			pool_id,
			SponsoredPoolJoinType::Default,
			long_url.clone(),
			account.clone(),
		);
		assert_err!(join, Error::<Test>::UrlTooLong);
	});
}
