
use crate::{mock::*};
use frame_support::{assert_err, assert_ok, traits::Currency};
use sp_runtime::AccountId32;

#[test]
fn verify_owner_works() {

    GameCreator::verify_owner();
}
