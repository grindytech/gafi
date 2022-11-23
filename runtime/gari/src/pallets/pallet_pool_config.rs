use frame_support::parameter_types;

use crate::{Balances, Event, Runtime, UpfrontPool, StakingPool, SponsoredPool, PalletCache};

parameter_types! {
	pub MaxJoinedSponsoredPool: u32 = 5;
	pub TimeServiceStorage: u128 = 30 * 60_000u128;
}

impl pallet_pool::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type UpfrontPool = UpfrontPool;
	type StakingPool = StakingPool;
	type WeightInfo = pallet_pool::weights::PoolWeight<Runtime>;
	type MaxJoinedSponsoredPool = MaxJoinedSponsoredPool;
	type SponsoredPool = SponsoredPool;
	type Cache = PalletCache;
	type TimeServiceStorage = TimeServiceStorage;
}
