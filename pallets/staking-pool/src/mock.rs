use crate as pallet_staking_pool;
use frame_support::{traits::{ConstU16, ConstU64, Hooks, GenesisBuild}, parameter_types};
use frame_system as system;
use gafi_primitives::option_pool::PackService;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup}, AccountId32,
};

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
		PalletPool: pallet_option_pool::{Pallet, Call, Storage, Event<T>},
		StakingPool: pallet_staking_pool::{Pallet, Call, Storage, Event<T>},
	}
);

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
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
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

impl pallet_staking_pool::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
	type OptionPool = PalletPool;
}


parameter_types! {
	pub const MaxNewPlayer: u32 = 1000;
	pub const MaxIngamePlayer: u32 = 1000;
}

impl pallet_option_pool::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type MaxNewPlayer = MaxNewPlayer;
	type MaxIngamePlayer = MaxIngamePlayer;
	type WeightInfo = ();
	type StakingPool = StakingPool;
}

pub const EXISTENTIAL_DEPOSIT: u64 = 1000;

parameter_types! {
	pub const ExistentialDeposit: u64 = EXISTENTIAL_DEPOSIT;
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


// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}


pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		StakingPool::on_initialize(System::block_number());
	}
}

pub const STAKE_AMOUNT: u128 = 100_000_000_000_000_000_000; // 100 AUX

pub struct ExtBuilder {
	staking_amount: u128,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			staking_amount: STAKE_AMOUNT,
		}
	}
}

impl ExtBuilder {
	fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let _ = pallet_balances::GenesisConfig::<Test> { balances: [].to_vec() }
			.assimilate_storage(&mut storage);

		let _ = pallet_staking_pool::GenesisConfig::<Test> {
			staking_amount: self.staking_amount,
			staking_discount: 50u8,
		}.assimilate_storage(&mut storage);

		// let _ = OtherGenesisConfig::<Test> {
		// 	max_player: 1000,
		// 	services: [
		// 		(PackService::Basic, 4, 60, 1),
		// 		(PackService::Medium, 8, 70, 1 * 2),
		// 		(PackService::Max, u8::MAX, 80, 1 * 3),
		// 	],
		// 	time_service: 3600,
		// }
		// .assimilate_storage(&mut storage);

		let mut ext = sp_io::TestExternalities::from(storage);
		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let mut ext = self.build();
		ext.execute_with(test);
		ext.execute_with(|| System::set_block_number(1));
	}
}
