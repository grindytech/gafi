use crate::{Balances, PalletCachePool, Runtime, FundingPool, StakingPool, UpfrontPool, RuntimeEvent};
use frame_support::parameter_types;
use gafi_primitives::{constant::ID, ticket::TicketInfo};

parameter_types! {
	pub CleanTime: u128 = 30 * 60_000u128; // 30 minutes;
}

// cache for pallet pool
impl pallet_cache::Config<pallet_cache::Instance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Data = TicketInfo;
	type Action = ID;
	type CleanTime = CleanTime;
}

parameter_types! {
	pub MaxJoinedFundingPool: u32 = 5;
	pub TimeServiceStorage: u128 = 30 * 60_000u128;
}

impl pallet_pool::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpfrontPool = UpfrontPool;
	type StakingPool = StakingPool;
	type WeightInfo = pallet_pool::weights::PoolWeight<Runtime>;
	type MaxJoinedFundingPool = MaxJoinedFundingPool;
	type FundingPool = FundingPool;
	type Cache = PalletCachePool;
	type TimeServiceStorage = TimeServiceStorage;
}
