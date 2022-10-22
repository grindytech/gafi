use sp_runtime::Permill;
use gafi_primitives::{
	currency::{unit, NativeToken::GAKI},
	system_services::{SystemDefaultServices, SystemService},
	constant::ID
};

pub const UPFRONT_BASIC_ID: ID = [10_u8; 32];
pub const UPFRONT_MEDIUM_ID: ID = [11_u8; 32];
pub const UPFRONT_ADVANCE_ID: ID = [12_u8; 32];

pub const STAKING_BASIC_ID: ID = [0_u8; 32];
pub const STAKING_MEDIUM_ID: ID = [1_u8; 32];
pub const STAKING_ADVANCE_ID: ID = [2_u8; 32];

pub struct UpfrontPoolDefaultServices {}

impl SystemDefaultServices for UpfrontPoolDefaultServices {
	fn get_default_services () -> [(ID, SystemService); 3] {
		[
			(
				UPFRONT_BASIC_ID,
				SystemService::new(UPFRONT_BASIC_ID, 10_u32, Permill::from_percent(30), 5 * unit(GAKI)),
			),
			(
				UPFRONT_MEDIUM_ID,
				SystemService::new(UPFRONT_MEDIUM_ID, 10_u32, Permill::from_percent(50), 7 * unit(GAKI)),
			),
			(
				UPFRONT_ADVANCE_ID,
				SystemService::new(UPFRONT_ADVANCE_ID, 10_u32, Permill::from_percent(70), 10 * unit(GAKI)),
			),
		]
	}
}

pub struct StakingPoolDefaultServices {}

impl SystemDefaultServices for StakingPoolDefaultServices {
	fn get_default_services () -> [(ID, SystemService); 3] {
		[
			(
				STAKING_BASIC_ID,
				SystemService::new(STAKING_BASIC_ID, 10_u32, Permill::from_percent(30), 1000 * unit(GAKI)),
			),
			(
				STAKING_MEDIUM_ID,
				SystemService::new(STAKING_MEDIUM_ID, 10_u32, Permill::from_percent(50), 1500 * unit(GAKI)),
			),
			(
				STAKING_ADVANCE_ID,
				SystemService::new(STAKING_ADVANCE_ID, 10_u32, Permill::from_percent(70), 2000 * unit(GAKI)),
			),
		]
	}
}
