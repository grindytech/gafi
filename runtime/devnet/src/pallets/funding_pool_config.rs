use frame_support::{parameter_types};
use frame_system::EnsureRoot;
use gafi_support::common::currency::{unit, NativeToken::GAKI};
use sp_runtime::{AccountId32, Permill};

use crate::{Balances, PalletWhitelist, RandomnessCollectiveFlip, Runtime, RuntimeEvent, Treasury};

parameter_types! {
	pub ReservationFee: u128 = 1 * unit(GAKI);
	pub MinLength: u32 = 8;
	pub MaxLength: u32 = 32;
}

impl pallet_nicks::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ReservationFee = ReservationFee;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRoot<AccountId32>;
	type MinLength = MinLength;
	type MaxLength = MaxLength;
}

parameter_types! {
	pub MinPoolBalance: u128 = 1000 * unit(GAKI);
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
	type Currency = Balances;
	type MinPoolBalance = MinPoolBalance;
	type MinDiscountPercent = MinDiscountPercent;
	type MaxDiscountPercent = MaxDiscountPercent;
	type MinTxLimit = MinTxLimit;
	type MaxTxLimit = MaxTxLimit;
	type MaxPoolOwned = MaxPoolOwned;
	type MaxPoolTarget = MaxPoolTarget;
	type IWhitelist = PalletWhitelist;
	type WeightInfo = funding_pool::weights::FundingWeight<Runtime>;
}
