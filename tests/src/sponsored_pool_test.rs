use crate::mock::*;
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	pool::{Level, TicketType, FlexPool},
};
use gafi_primitives::constant::ID;
use gafi_primitives::pool::{StaticPool};
use sp_runtime::AccountId32;
use sp_core::H160;
use sp_std::str::FromStr;
use sp_std::vec::{Vec};

fn new_account(account: [u8; 32], balance: u128) -> AccountId32 {
    let acc: AccountId32 = AccountId32::from(account.clone());
    let _ = pallet_balances::Pallet::<Test>::deposit_creating(&acc, balance);
    assert_eq!(Balances::free_balance(&acc), balance);
    return acc;
}

fn create_pool(
    account: AccountId32,
    targets: Vec<H160>,
    pool_value: u128,
    tx_limit: u32,
    discount: u8,
) {

    {
        let before_balance = Balances::free_balance(&account);
        assert_ok!(SponsoredPool::create_pool(
            Origin::signed(account.clone()),
            targets,
            pool_value,
            discount,
            tx_limit
        ));
        assert_eq!(
            Balances::free_balance(&account),
            before_balance - pool_value
        );
    }
}

#[test]
fn join_upfront_pool_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(1);
        let account_balance = 1_000_000 * unit(GAKI);
        let account = new_account([0_u8; 32], account_balance);
        let targets = vec![H160::default()];
        let pool_value = 1000 * unit(GAKI);
        let tx_limit = 100_u32;
        let discount = 30_u8;

        create_pool(account, targets, pool_value, tx_limit, discount)
    })
}