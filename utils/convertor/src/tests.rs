use crate::*;
use frame_support::{assert_ok};
use frame_support::traits::Currency;
use sp_runtime::{
    AccountId32,
};
use mock::*;
use gu_dummy::Config;

#[test]
fn u128_try_to_balance_works() {

   assert_ok!(u128_try_to_balance::<<Test as Config>::Currency, AccountId32>(1_000_000));

}
