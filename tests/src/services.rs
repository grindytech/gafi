use crate::mock::*;
use aurora_primitives::{centi, currency::NativeToken::AUX, unit};
use frame_support::{assert_err, assert_ok, traits::Currency};
use hex_literal::hex;
use pallet_pool::pool::PackService;
use pallet_tx_handler::Config;
use sp_core::H160;
use sp_runtime::AccountId32;
use sp_std::str::FromStr;

const CIRCLE_BLOCK: u64 = (TIME_SERVICE as u64) / SLOT_DURATION;
const ADDITIONAL_BLOCK: u64 = 1;

fn init_join_pool(pool_fee: u128, pack: PackService, is_bond: bool) {
	let sender = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap(); //ALICE

	let base_balance = 1000 * unit(AUX);
	let _ = <Test as Config>::Currency::deposit_creating(&sender, base_balance);
	{
		assert_eq!(<Test as Config>::Currency::free_balance(sender.clone()), base_balance);
	}

	if is_bond {
		let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();
		assert_ok!(PalletTxHandler::bond(
			Origin::signed(sender.clone()),
			signature,
			address,
			false
		));
	}
	assert_ok!(PalletPool::join(Origin::signed(sender.clone()), pack));
	assert_eq!(
		<Test as Config>::Currency::free_balance(sender.clone()),
		base_balance - (pool_fee * 2)
	); //charge x2 when once join

    for i in 1 .. 10 {
        run_to_block((CIRCLE_BLOCK * i) + ADDITIONAL_BLOCK);
        // let _now = pallet_timestamp::Pallet::<Test>::get();
        assert_eq!(<Test as Config>::Currency::free_balance(sender.clone()), base_balance - (pool_fee * (i + 1) as u128));
    }
}

#[test]
fn charge_join_pool_basic_work() {
	ExtBuilder::default().build_and_execute(|| {
        let pool_fee: u128 = 75 * centi(AUX); // 0.75 AUX
		init_join_pool(pool_fee, PackService::Basic, false);
    })
}

#[test]
fn charge_join_pool_medium_work() {
	ExtBuilder::default().build_and_execute(|| {
        let pool_fee: u128 = 75 * centi(AUX); // 0.75 AUX
        let pool_fee = pool_fee * 2;
		
        init_join_pool(pool_fee, PackService::Medium, true);
    })
}

#[test]
fn charge_join_max_pool_work() {
	ExtBuilder::default().build_and_execute(|| {
        let pool_fee: u128 = 75 * centi(AUX); // 0.75 AUX
        let pool_fee = pool_fee * 3;
		
        init_join_pool(pool_fee, PackService::Max, true);
    })
}
