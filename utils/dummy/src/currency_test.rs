use crate::*;
use crate::mock::*;
use sp_runtime::AccountId32;
use gu_currency::transfer_all;

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
fn transfer_all_keep_alive_works() {
    new_test_ext().execute_with(|| {
		const BALANCE: u128 = 1_000_000_000;
		
        let account_1 = new_account([0_u8; 32], BALANCE);
        let account_2 = new_account([1_u8; 32], BALANCE);
		

		let _ = transfer_all::<Test, <Test as pallet::Config>::Currency>(&account_1, &account_2, true);

		// evm_address balance should  
		{
			assert_eq!(Balances::free_balance(&account_1), EXISTENTIAL_DEPOSIT);
			assert_eq!(Balances::free_balance(&account_2), BALANCE + BALANCE - EXISTENTIAL_DEPOSIT);
		}
    })
}


#[test]
fn transfer_all_allow_death_works() {
    new_test_ext().execute_with(|| {
		const BALANCE: u128 = 1_000_000_000;
		
        let account_1 = new_account([0_u8; 32], BALANCE);
        let account_2 = new_account([1_u8; 32], BALANCE);

		let _ = transfer_all::<Test, <Test as pallet::Config>::Currency>(&account_1, &account_2, false);

		{
			assert_eq!(Balances::free_balance(&account_1), 0_u128);
			assert_eq!(Balances::free_balance(&account_2), BALANCE + BALANCE);
		}
    })
}
