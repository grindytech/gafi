use frame_support::parameter_types;
use gafi_primitives::currency::{unit, NativeToken::GAFI};
use sp_runtime::Permill;

use crate::{Balances, PoolName, RandomnessCollectiveFlip, Runtime, RuntimeEvent};

parameter_types! {
	pub MinPoolBalance: u128 = 1000 * unit(GAFI);
	pub MinDiscountPercent: Permill = Permill::from_percent(10);
	pub MaxDiscountPercent: Permill = Permill::from_percent(100);
	pub MinTxLimit: u32 = 30;
	pub MaxTxLimit: u32 = 100;
	pub MaxPoolOwned: u32 =  10;
	pub MaxPoolTarget: u32 =  10;
}

impl funding_pool::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Randomness = RandomnessCollectiveFlip;
	type PoolName = PoolName;
	type Currency = Balances;
	type MinPoolBalance = MinPoolBalance;
	type MinDiscountPercent = MinDiscountPercent;
	type MaxDiscountPercent = MaxDiscountPercent;
	type MinTxLimit = MinTxLimit;
	type MaxTxLimit = MaxTxLimit;
	type MaxPoolOwned = MaxPoolOwned;
	type MaxPoolTarget = MaxPoolTarget;
	type IWhitelist = ();
	type WeightInfo = funding_pool::weights::FundingWeight<Runtime>;
}
