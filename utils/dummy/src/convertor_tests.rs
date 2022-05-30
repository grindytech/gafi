use crate::*;
use frame_support::assert_ok;
use frame_support::traits::Currency;
use gu_dummy::Config;
use crate::mock::*;
use sp_runtime::AccountId32;
use gu_convertor::{balance_try_to_u128, into_account, u128_to_balance, u128_try_to_balance};

#[test]
fn u128_and_balance_convertor_works() {
    {
        let balance =
            u128_try_to_balance::<<Test as Config>::Currency, AccountId32>(1000_000_000_u128)
                .unwrap();
        let balance_amount =
            balance_try_to_u128::<<Test as Config>::Currency, AccountId32>(balance).unwrap();
        assert_eq!(1000_000_000_u128, balance_amount);
    }

    {
        let balance =
            u128_try_to_balance::<<Test as Config>::Currency, AccountId32>(std::u128::MIN).unwrap();
        let balance_amount =
            balance_try_to_u128::<<Test as Config>::Currency, AccountId32>(balance).unwrap();
        assert_eq!(std::u128::MIN, balance_amount);
    }

    {
        let balance =
            u128_try_to_balance::<<Test as Config>::Currency, AccountId32>(std::u128::MAX).unwrap();
        let balance_amount =
            balance_try_to_u128::<<Test as Config>::Currency, AccountId32>(balance).unwrap();
        assert_eq!(std::u128::MAX, balance_amount);
    }
}

#[test]
fn u128_to_balance_works() {
    {
        let balance = u128_to_balance::<<Test as Config>::Currency, AccountId32>(1000_000_000_u128);
        let balance_amount =
            balance_try_to_u128::<<Test as Config>::Currency, AccountId32>(balance).unwrap();
        assert_eq!(1000_000_000_u128, balance_amount);
    }

    {
        let balance =
            u128_to_balance::<<Test as Config>::Currency, AccountId32>(std::u128::MIN);
        let balance_amount =
            balance_try_to_u128::<<Test as Config>::Currency, AccountId32>(balance).unwrap();
        assert_eq!(std::u128::MIN, balance_amount);
    }

    {
        let balance =
            u128_to_balance::<<Test as Config>::Currency, AccountId32>(std::u128::MAX);
        let balance_amount =
            balance_try_to_u128::<<Test as Config>::Currency, AccountId32>(balance).unwrap();
        assert_eq!(std::u128::MAX, balance_amount);
    }
}

#[test]
fn into_account_works() {
    let account = into_account::<AccountId32>([0_u8; 32]).unwrap();
    assert_eq!(account, AccountId32::from([0_u8; 32]));
}
