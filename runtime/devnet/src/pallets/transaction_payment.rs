use frame_support::{traits::ConstU8, weights::IdentityFee};
use pallet_transaction_payment::CurrencyAdapter;
use runtime_common::impls::DealWithFees;

use crate::{Balance, Balances, RuntimeEvent, Runtime};

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = crate::RuntimeEvent;
	type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees<Runtime>>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}
