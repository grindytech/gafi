use frame_support::parameter_types;
use gafi_primitives::{
	constant::ID,
	currency::{unit, NativeToken::GAKI},
	system_services::{SystemDefaultServices, SystemService},
};
use sp_runtime::Permill;

use crate::{Balances, Event, Player, Pool, Runtime};

const UPFRONT_BASIC_ID: ID = [10_u8; 32];
const UPFRONT_MEDIUM_ID: ID = [11_u8; 32];
const UPFRONT_ADVANCE_ID: ID = [12_u8; 32];
pub struct UpfrontPoolDefaultServices {}

impl SystemDefaultServices for UpfrontPoolDefaultServices {
	fn get_default_services() -> [(ID, SystemService); 3] {
		[
			(
				UPFRONT_BASIC_ID,
				SystemService::new(
					UPFRONT_BASIC_ID,
					10_u32,
					Permill::from_percent(30),
					5 * unit(GAKI),
				),
			),
			(
				UPFRONT_MEDIUM_ID,
				SystemService::new(
					UPFRONT_MEDIUM_ID,
					10_u32,
					Permill::from_percent(50),
					7 * unit(GAKI),
				),
			),
			(
				UPFRONT_ADVANCE_ID,
				SystemService::new(
					UPFRONT_ADVANCE_ID,
					10_u32,
					Permill::from_percent(70),
					10 * unit(GAKI),
				),
			),
		]
	}
}

parameter_types! {
	pub const MaxPlayerStorage: u32 = 10000;
}

impl upfront_pool::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = upfront_pool::weights::SubstrateWeight<Runtime>;
	type MaxPlayerStorage = MaxPlayerStorage;
	type MasterPool = Pool;
	type UpfrontServices = UpfrontPoolDefaultServices;
	type Players = Player;
}
