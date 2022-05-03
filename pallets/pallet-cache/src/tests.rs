use crate::{mock::*, DataFlag, DataLeft, DataRight, Error, Flag, Pallet};
use frame_support::{assert_err, assert_ok, traits::Currency};
use gafi_primitives::cache::Cache;
use gafi_primitives::{
    currency::{unit, NativeToken::GAKI},
    player::TicketInfo,
    pool::{Level, TicketType},
};
use sp_runtime::AccountId32;
use sp_std::str::FromStr;
const CIRCLE_BLOCK: u64 = (TIME_SERVICE as u64) / SLOT_DURATION;
const ADDL_BLOCK: u64 = 1_u64;

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
fn insert_data_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(1);
        let account = new_account([0_u8; 32], 1_000_000_u128 * unit(GAKI));
        let data = TicketInfo {
            ticket_type: TicketType::Upfront(Level::Basic),
            tickets: 100_u32,
        };
        Pallet::<Test>::insert(&account, data.ticket_type, data);

        if DataFlag::<Test>::get() == Flag::Left {
            assert_eq!(
                DataLeft::<Test>::get(account.clone(), data.ticket_type)
                    .unwrap()
                    .data,
                data
            );
        }
    })
}

#[test]
fn get_data_insert_early_work() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(ADDL_BLOCK);
        let account = new_account([0_u8; 32], 1_000_000_u128 * unit(GAKI));
        let data = TicketInfo {
            ticket_type: TicketType::Upfront(Level::Basic),
            tickets: 100_u32,
        };
        Pallet::<Test>::insert(&account, data.ticket_type, data);

        run_to_block(CIRCLE_BLOCK + ADDL_BLOCK);
        assert_eq!(Pallet::<Test>::get(&account, data.ticket_type), None);

        run_to_block(CIRCLE_BLOCK * 2 + ADDL_BLOCK);
        assert_eq!(Pallet::<Test>::get(&account, data.ticket_type), None);
    })
}

#[test]
fn get_data_insert_late_work() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(CIRCLE_BLOCK - ADDL_BLOCK);
        let account = new_account([0_u8; 32], 1_000_000_u128 * unit(GAKI));
        let data = TicketInfo {
            ticket_type: TicketType::Upfront(Level::Basic),
            tickets: 100_u32,
        };
        Pallet::<Test>::insert(&account, data.ticket_type, data);

        run_to_block(CIRCLE_BLOCK + ADDL_BLOCK);
        assert_eq!(Pallet::<Test>::get(&account, data.ticket_type), Some(data));

        run_to_block(CIRCLE_BLOCK * 2 + ADDL_BLOCK);
        assert_eq!(Pallet::<Test>::get(&account, data.ticket_type), None);
    })
}
