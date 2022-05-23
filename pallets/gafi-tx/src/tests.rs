use crate::{mock::*, Config, Error, Pallet};
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::constant::ID;
use gafi_primitives::currency::{unit, NativeToken::GAKI};
use pallet_ethereum::RawOrigin;
use pallet_evm::AddressMapping;
use pallet_evm::{ExitReason, ExitSucceed, Runner};
use sp_core::{
    bytes::{from_hex, to_hex},
    H160, U256,
};
use sp_runtime::AccountId32;
use sp_std::str::FromStr;

fn make_deposit(account: &AccountId32, balance: u128) {
    let _ = pallet_balances::Pallet::<Test>::deposit_creating(account, balance);
}

fn new_account(account: [u8; 32], balance: u128) -> AccountId32 {
    let acc: AccountId32 = AccountId32::from(account);
    make_deposit(&acc, balance);
    assert_eq!(Balances::free_balance(&acc), balance);
    return acc;
}

#[test]
fn correct_and_deposit_fee_sponsored_works() {
    ExtBuilder::default().build_and_execute(|| {
        let pool_id: ID = [0_u8; 32];
        let pool = AccountId32::from(pool_id);
        let pool_balance = 100 * unit(GAKI);
        let service_fee = 10 * unit(GAKI);
        make_deposit(&pool, pool_balance);

        let targets = vec![H160::from_str("0x0A6617b82B594C83240092BDc86E2e16354d1456").unwrap()];
        let target: H160 = H160::from_str("0x0A6617b82B594C83240092BDc86E2e16354d1456").unwrap();
        let discount = 40_u8;

        let sponsored_fee = Pallet::<Test>::correct_and_deposit_fee_sponsored(
            pool_id,
            targets,
            target,
            U256::from(service_fee),
            discount,
        )
        .unwrap();

        assert_eq!(sponsored_fee, U256::from(6 * unit(GAKI)));
        assert_eq!(Balances::free_balance(&pool), 96 * unit(GAKI));
    })
}
