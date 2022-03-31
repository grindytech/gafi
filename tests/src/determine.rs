use aurora_primitives::{currency::NativeToken::AUX, unit};
use hex_literal::hex;
use frame_support::{assert_err, assert_ok, traits::Currency};
use pallet_tx_handler::Config;
use sp_core::H160;
use sp_runtime::AccountId32;
use crate::{mock::*};
use sp_std::str::FromStr;



const CIRCLE_BLOCK: u128 = TIME_SERVICE / SLOT_DURATION as u128;

#[test]
fn determine_basic() {
    ExtBuilder::default().build_and_execute(|| {
		let sender = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap(); //ALICE
		let signature: [u8; 65] = hex!("20b4f726ffe9333370c64dba5bb50b01e84e1bc8d05b7be0fa8a7c52fcd5c3f46ef44800722a545ad70b8da26fea9cf80fba72a65bb119c7a93e81c3e51edf501b");
		let address: H160 = H160::from_str("b28049C6EE4F90AE804C70F860e55459E837E84b").unwrap();

        let base_balance = 1000 * unit(AUX);
		let _ = <Test as Config>::Currency::deposit_creating(&sender, base_balance);
        {
            assert_eq!(<Test as Config>::Currency::free_balance(sender.clone()), base_balance);
        }

        assert_ok!(PalletTxHandler::bond(Origin::signed(sender.clone()), signature, address, false));

        run_to_block(10);

        assert_eq!(<Test as Config>::Currency::free_balance(sender.clone()), base_balance);

    })
}
