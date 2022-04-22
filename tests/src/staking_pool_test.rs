use crate::mock::*;
use frame_support::{assert_ok, traits::Currency};
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	pool::{GafiPool, Level, TicketType},
};
use gafi_tx::Config;
use sp_runtime::AccountId32;

const LEVELS: [Level; 3] = [Level::Basic, Level::Medium, Level::Advance];

fn join_pool(account: AccountId32, staking_amount: u128, ticket: TicketType) {
	let base_balance = 1_000_000 * unit(GAKI);
	let _ = <Test as Config>::Currency::deposit_creating(&account, base_balance);
	{
		assert_eq!(<Test as Config>::Currency::free_balance(account.clone()), base_balance);
	}

	assert_ok!(Pool::join(Origin::signed(account.clone()), ticket));
	assert_eq!(
		<Test as Config>::Currency::free_balance(account.clone()),
		base_balance - staking_amount 
	);
}

fn leave_pool(account: AccountId32, staking_amount: u128) {
    let before_balance = <Test as Config>::Currency::free_balance(account.clone());
	assert_ok!(Pool::leave(Origin::signed(account.clone())));
	assert_eq!(
		<Test as Config>::Currency::free_balance(account.clone()),
		before_balance + staking_amount 
	);
}

#[test]
fn join_pool_works() {
    for i in 0..LEVELS.len() {
        ExtBuilder::default().build_and_execute(|| {
            let pool_fee = StakingPool::get_service(LEVELS[i]);
            let account = AccountId32::new([i as u8; 32]);
            join_pool(account, pool_fee.value, TicketType::Staking(LEVELS[i]));
        })
    }
}


#[test]
fn leave_pool_works() {
    for i in 0..LEVELS.len() {
        ExtBuilder::default().build_and_execute(|| {
            let pool_fee = StakingPool::get_service(LEVELS[i]);
            let account = AccountId32::new([i as u8; 32]);
            join_pool(account.clone(), pool_fee.value, TicketType::Staking(LEVELS[i]));
            leave_pool(account.clone(), pool_fee.value);
        })
    }
}