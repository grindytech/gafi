/*
* This unittest should only test logic function e.g. Storage, Computation
* and not related with Currency e.g. Balances, Transaction Payment
*/

use crate::{self as pallet_option_pool};
use gafi_primitives::option_pool::PackService;
use frame_support::parameter_types;
use frame_system as system;

use frame_support::{
	dispatch::Vec,
	traits::{Currency, OnFinalize, OnInitialize},
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};

pub use pallet_balances::Call as BalancesCall;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const TEST_ACCOUNTS: [(AccountId32, u64); 10] = [
	(AccountId32::new([0u8; 32]), 1000000000000000000),
	(AccountId32::new([1u8; 32]), 1000000000000000000),
	(AccountId32::new([2u8; 32]), 1000000000000000000),
	(AccountId32::new([3u8; 32]), 1000000000000000000),
	(AccountId32::new([4u8; 32]), 1000000000000000000),
	(AccountId32::new([5u8; 32]), 1000000000000000000),
	(AccountId32::new([6u8; 32]), 1000000000000000000),
	(AccountId32::new([7u8; 32]), 1000000000000000000),
	(AccountId32::new([8u8; 32]), 1000000000000000000),
	(AccountId32::new([9u8; 32]), 1000000000000000000),
];

const POOL_FEE: u64 = 10000000000000000;
pub const MAX_PLAYER: u32 = 20;
pub const MAX_NEW_PLAYER: u32 = 20;
pub const MAX_INGAME_PLAYER: u32 = 20;
pub const TIME_SERVICE: u128 =  60_000u128; // 10 second

pub const SERVICES: [(PackService, u8, u8, u64); 3] = [
	(PackService::Basic, 4, 60, POOL_FEE),
	(PackService::Medium, 8, 70, POOL_FEE * 2),
	(PackService::Max, u8::MAX, 80, POOL_FEE * 3),
];

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		PalletPool: pallet_option_pool::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		// Event: Event,
	}
);

impl pallet_randomness_collective_flip::Config for Test {}

pub const EXISTENTIAL_DEPOSIT: u64 = 1000;

parameter_types! {
	pub ExistentialDeposit: u64 = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
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
	type AccountData = pallet_balances::AccountData<u64>;
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

parameter_types! {
	pub const MaxNewPlayer: u32 = MAX_NEW_PLAYER;
	pub const MaxIngamePlayer: u32 = MAX_INGAME_PLAYER;
}

impl pallet_option_pool::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type MaxNewPlayer = MaxNewPlayer;
	type MaxIngamePlayer = MaxIngamePlayer;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			PalletPool::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		PalletPool::on_initialize(System::block_number());
		Timestamp::set_timestamp((System::block_number() as u64 * MILLISECS_PER_BLOCK) + INIT_TIMESTAMP);
	}
}

pub struct ExtBuilder {
	balances: Vec<(AccountId32, u64)>,
	max_player: u32,
	services: [(PackService, u8, u8, u64); 3],
	time_service: u128,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: TEST_ACCOUNTS.to_vec(),
			max_player: MAX_PLAYER,
			services: SERVICES,
			time_service: TIME_SERVICE,
		}
	}
}

impl ExtBuilder {
	fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let _ = pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut storage);

		let _ = pallet_option_pool::GenesisConfig::<Test> {
			max_player: self.max_player,
			services: self.services,
			time_service: self.time_service,
		}
		.assimilate_storage(&mut storage);

		let mut ext = sp_io::TestExternalities::from(storage);
		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let mut ext = self.build();
		ext.execute_with(test);
		ext.execute_with(|| System::set_block_number(1));
	}
}
