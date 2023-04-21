use frame_support::parameter_types;

use crate::{Balances, ProofAddressMapping, Runtime, EVM, RuntimeEvent};
use gafi_support::common::currency::{unit, NativeToken::GAFI};

parameter_types! {
	pub MaxContractOwned: u32 = 1000;
	pub GameCreatorFee: u128 = 5 * unit(GAFI);
}

impl game_creator::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AddressMapping = ProofAddressMapping;
	type MaxContractOwned = MaxContractOwned;
	type ContractCreator = EVM;
	type ReservationFee = GameCreatorFee;
	type WeightInfo = game_creator::weights::GameCreatorWeight<Runtime>;
}
