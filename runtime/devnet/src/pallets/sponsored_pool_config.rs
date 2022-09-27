use frame_support::parameter_types;
use gafi_primitives::currency::{unit, NativeToken::GAKI};
use sp_runtime::Permill;

use crate::{Balances, Event, PalletWhitelist, PoolName, RandomnessCollectiveFlip, Runtime};

parameter_types! {
	pub MinPoolBalance: u128 = 1000 * unit(GAKI);
	pub MinDiscountPercent: Permill = Permill::from_percent(30);
	pub MaxDiscountPercent: Permill = Permill::from_percent(70);
	pub MinTxLimit: u32 = 50;
	pub MaxTxLimit: u32 = 100;
	pub MaxPoolOwned: u32 =  10;
	pub MaxPoolTarget: u32 =  10;
}

impl sponsored_pool::Config for Runtime {
	type Event = Event;
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
	type IWhitelist = PalletWhitelist;
	type WeightInfo = sponsored_pool::weights::SponsoredWeight<Runtime>;
}
