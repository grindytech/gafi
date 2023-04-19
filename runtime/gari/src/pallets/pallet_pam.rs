use crate::{Balances, Runtime, RuntimeEvent};
use frame_support::parameter_types;
use gafi_primitives::common::currency::{unit, NativeToken::GAFI};

use proof_address_mapping;

parameter_types! {
	pub Prefix: &'static [u8] =  b"Bond Gafi Network account:";
	pub Fee: u128 = 1 * unit(GAFI);
}

impl proof_address_mapping::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = proof_address_mapping::weights::SubstrateWeight<Runtime>;
	type MessagePrefix = Prefix;
	type ReservationFee = Fee;
}

