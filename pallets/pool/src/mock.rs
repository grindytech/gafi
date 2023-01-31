/*
 * This unittest should only test logic function e.g. Storage, Computation
 * and not related with Currency e.g. Balances, Transaction Payment
 */

use crate::{self as pallet_pool};
use frame_support::{
	parameter_types,
	traits::{ConstU32, GenesisBuild},
};
use frame_system as system;

use frame_support::{
	dispatch::Vec,
	traits::{OnFinalize, OnInitialize},
};
use gafi_primitives::{
	constant::ID,
	currency::{unit, NativeToken::GAKI},
	ticket::TicketInfo,
};
pub use gu_mock::{pool::*, AN_HOUR};
pub use pallet_balances::Call as BalancesCall;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, Permill,
};

pub use staking_pool;
pub use upfront_pool;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Pool: pallet_pool::{Pallet, Storage, Event<T>},
		StakingPool: staking_pool::{Pallet, Storage, Event<T>},
		UpfrontPool: upfront_pool::{Pallet, Call, Storage, Event<T>},
		FundingPool: funding_pool::{Pallet, Call, Storage, Event<T>},
		PoolNames: pallet_pool_names::{Pallet, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		PalletCache: pallet_cache::{Pallet, Storage, Event<T>}
	}
);

pub const EXISTENTIAL_DEPOSIT: u128 = 1000;

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u128;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const INIT_TIMESTAMP: u64 = 30_000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

pub const RESERVATION_FEE: u128 = 2;

parameter_types! {
	pub ReservationFee: u128 = RESERVATION_FEE * unit(GAKI);
}
impl pallet_pool_names::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type ReservationFee = ReservationFee;
	type Slashed = ();
	type MinLength = ConstU32<3>;
	type MaxLength = ConstU32<16>;
}

parameter_types! {
	pub MinPoolBalance: u128 = 1000 * unit(GAKI);
	pub MinDiscountPercent: Permill = Permill::from_percent(10);
	pub MaxDiscountPercent: Permill = Permill::from_percent(70);
	pub MinTxLimit: u32 = 10;
	pub MaxTxLimit: u32 = 100;
	pub MaxPoolOwned: u32 =  10;
	pub MaxPoolTarget: u32 =  10;
}

impl funding_pool::Config for Test {
	type Event = Event;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type PoolName = PoolNames;
	type MaxPoolOwned = MaxPoolOwned;
	type MaxPoolTarget = MaxPoolTarget;
	type MinDiscountPercent = MinDiscountPercent;
	type MaxDiscountPercent = MaxDiscountPercent;
	type MinTxLimit = MinTxLimit;
	type MaxTxLimit = MaxTxLimit;
	type MinPoolBalance = MinPoolBalance;
	type WeightInfo = ();
	type IWhitelist = ();
}

parameter_types! {
	pub CleanTime: u128 = AN_HOUR;
}

impl pallet_cache::Config for Test {
	type Event = Event;
	type Data = TicketInfo;
	type Action = ID;
	type CleanTime = CleanTime;
}

parameter_types! {
	pub MaxJoinedFundingPool: u32 = 5;
	pub TimeServiceStorage: u128 = 30 * 60_000u128;
}

impl pallet_pool::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type UpfrontPool = UpfrontPool;
	type StakingPool = StakingPool;
	type WeightInfo = ();
	type MaxJoinedFundingPool = MaxJoinedFundingPool;
	type FundingPool = FundingPool;
	type Cache = PalletCache;
	type TimeServiceStorage = TimeServiceStorage;
}

impl staking_pool::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
	type StakingServices = StakingPoolDefaultServices;
	type Players = ();
}

parameter_types! {
	pub MaxPlayerStorage: u32 = 1000;
}

impl upfront_pool::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
	type MaxPlayerStorage = MaxPlayerStorage;
	type MasterPool = ();
	type UpfrontServices = UpfrontPoolDefaultServices;
	type Players = ();
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 24;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type AccountData = pallet_balances::AccountData<u128>;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// Build genesis storage according to the mock runtime.
pub fn _new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	GenesisBuild::<Test>::assimilate_storage(&upfront_pool::GenesisConfig::default(), &mut storage)
		.unwrap();

	GenesisBuild::<Test>::assimilate_storage(&staking_pool::GenesisConfig::default(), &mut storage)
		.unwrap();

	let ext = sp_io::TestExternalities::from(storage);
	ext
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Timestamp::set_timestamp(
			(System::block_number() as u64 * MILLISECS_PER_BLOCK) + INIT_TIMESTAMP,
		);
	}
}

pub struct ExtBuilder {
	pub time_service: u128,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			time_service: AN_HOUR,
		}
	}
}

impl ExtBuilder {
	fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		GenesisBuild::<Test>::assimilate_storage(
			&upfront_pool::GenesisConfig::default(),
			&mut storage,
		)
		.unwrap();

		GenesisBuild::<Test>::assimilate_storage(
			&staking_pool::GenesisConfig::default(),
			&mut storage,
		)
		.unwrap();

		let ext = sp_io::TestExternalities::from(storage);
		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let mut ext = self.build();
		ext.execute_with(test);
		ext.execute_with(|| System::set_block_number(1));
	}
}