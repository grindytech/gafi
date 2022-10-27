use gafi_primitives::{
	constant::ID,
	currency::{unit, NativeToken::GAKI},
	system_services::{SystemDefaultServices, SystemService},
};
use sp_runtime::Permill;

use crate::{Balances, Event, Player, Runtime};

const STAKING_BASIC_ID: ID = [0_u8; 32];
const STAKING_MEDIUM_ID: ID = [1_u8; 32];
const STAKING_ADVANCE_ID: ID = [2_u8; 32];

impl SystemDefaultServices for StakingPoolDefaultServices {
	fn get_default_services() -> [(ID, SystemService); 3] {
		[
			(
				STAKING_BASIC_ID,
				SystemService::new(
					STAKING_BASIC_ID,
					100_u32,
					Permill::from_percent(10),
					1000 * unit(GAKI),
				),
			),
			(
				STAKING_MEDIUM_ID,
				SystemService::new(
					STAKING_MEDIUM_ID,
					100_u32,
					Permill::from_percent(20),
					2000 * unit(GAKI),
				),
			),
			(
				STAKING_ADVANCE_ID,
				SystemService::new(
					STAKING_ADVANCE_ID,
					100_u32,
					Permill::from_percent(30),
					3000 * unit(GAKI),
				),
			),
		]
	}
}

pub struct StakingPoolDefaultServices {}

impl staking_pool::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = staking_pool::weights::SubstrateWeight<Runtime>;
	type StakingServices = StakingPoolDefaultServices;
	type Players = Player;
}