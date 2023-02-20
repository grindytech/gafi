use frame_support::parameter_types;
use gafi_primitives::{
	constant::ID,
	currency::{unit, NativeToken::GAFI},
	system_services::{SystemDefaultServices, SystemService, SystemServicePack},
};
use sp_runtime::Permill;
use codec::{Encode, Decode};
use sp_std::vec;

use crate::{Balances, Player, Pool, Runtime, RuntimeEvent};

const UPFRONT_BASIC_ID: ID = [10_u8; 32];
const UPFRONT_MEDIUM_ID: ID = [11_u8; 32];
const UPFRONT_ADVANCE_ID: ID = [12_u8; 32];

#[derive(Eq, PartialEq, Clone, Encode, Decode)]
pub struct UpfrontPoolDefaultServices {}

impl SystemDefaultServices for UpfrontPoolDefaultServices {
	fn get_default_services() -> SystemServicePack {
		SystemServicePack::new(vec![
			(
				UPFRONT_BASIC_ID,
				SystemService::new(
					UPFRONT_BASIC_ID,
					100_u32,
					Permill::from_percent(30),
					15 * unit(GAFI),
				),
			),
			(
				UPFRONT_MEDIUM_ID,
				SystemService::new(
					UPFRONT_MEDIUM_ID,
					100_u32,
					Permill::from_percent(40),
					20 * unit(GAFI),
				),
			),
			(
				UPFRONT_ADVANCE_ID,
				SystemService::new(
					UPFRONT_ADVANCE_ID,
					100_u32,
					Permill::from_percent(50),
					25 * unit(GAFI),
				),
			),
		])
	}
}

parameter_types! {
	pub const MaxPlayerStorage: u32 = 10000;
}

impl upfront_pool::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = upfront_pool::weights::SubstrateWeight<Runtime>;
	type MaxPlayerStorage = MaxPlayerStorage;
	type MasterPool = Pool;
	type UpfrontServices = UpfrontPoolDefaultServices;
	type Players = Player;
}
