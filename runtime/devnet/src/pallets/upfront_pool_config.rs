use codec::{Decode, Encode};
use frame_support::parameter_types;
pub use gafi_support::{
	common::{
		constant::ID,
		currency::{centi, deposit, microcent, milli, unit, NativeToken::GAKI},
	},
	pallet::{cache::Cache, players::PlayerJoinedPoolStatistic},
	pool::{
		system_services::{SystemDefaultServices, SystemService, SystemServicePack},
		ticket::{TicketInfo, TicketType},

	},
};
use sp_runtime::Permill;
use sp_std::vec;

use crate::{Balances, Pool, Runtime};

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
					15 * unit(GAKI),
				),
			),
			(
				UPFRONT_MEDIUM_ID,
				SystemService::new(
					UPFRONT_MEDIUM_ID,
					100_u32,
					Permill::from_percent(40),
					20 * unit(GAKI),
				),
			),
			(
				UPFRONT_ADVANCE_ID,
				SystemService::new(
					UPFRONT_ADVANCE_ID,
					100_u32,
					Permill::from_percent(50),
					25 * unit(GAKI),
				),
			),
		])
	}
}

parameter_types! {
	pub const MaxPlayerStorage: u32 = 10000;
}

impl upfront_pool::Config for Runtime {
	type RuntimeEvent = crate::RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = upfront_pool::weights::SubstrateWeight<Runtime>;
	type MaxPlayerStorage = MaxPlayerStorage;
	type MasterPool = Pool;
	type UpfrontServices = UpfrontPoolDefaultServices;
}
