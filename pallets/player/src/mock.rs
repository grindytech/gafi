use crate as pallet_player;
use codec::Encode;
use frame_support::parameter_types;
use frame_system as system;

use frame_support::traits::{OnFinalize, OnInitialize, GenesisBuild};
use gafi_primitives::{
	system_services::{SystemDefaultServices, SystemService},
	constant::ID,
	ticket::{TicketLevel, SystemTicket},
	currency::{unit, NativeToken::GAKI}
};
use sp_core::{H256, blake2_256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, Permill,
};

pub use pallet_balances::Call as BalancesCall;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		PalletGame: pallet_player::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		UpfrontPool: upfront_pool::{Pallet, Call, Storage, Event<T>}
		// Event: Event,
	}
);

impl pallet_randomness_collective_flip::Config for Test {}

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
	pub const ExistentialDeposit: u64 = 1;
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

pub struct UpfrontPoolDefaultServices {}

impl SystemDefaultServices for UpfrontPoolDefaultServices {
	fn get_default_services () -> [(ID, SystemService); 3] {
		[
			(
				(SystemTicket::Upfront(TicketLevel::Basic)).using_encoded(blake2_256),
				SystemService::new(TicketLevel::Basic, 10_u32, Permill::from_percent(30), 5 * unit(GAKI)),
			),
			(
				(SystemTicket::Upfront(TicketLevel::Medium)).using_encoded(blake2_256),
				SystemService::new(TicketLevel::Medium, 10_u32, Permill::from_percent(50), 7 * unit(GAKI)),
			),
			(
				(SystemTicket::Upfront(TicketLevel::Advance)).using_encoded(blake2_256),
				SystemService::new(TicketLevel::Advance, 10_u32, Permill::from_percent(70), 10 * unit(GAKI)),
			),
		]
	}
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
	type Players = PalletGame;
}

impl pallet_player::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type GameRandomness = RandomnessCollectiveFlip;
	type UpfrontPool = UpfrontPool;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

	GenesisBuild::<Test>::assimilate_storage(
		&upfront_pool::GenesisConfig::default(),
		&mut storage,
	)
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
		Timestamp::set_timestamp((System::block_number() as u64 * MILLISECS_PER_BLOCK) + INIT_TIMESTAMP);
	}
}

// pub struct ExtBuilder;

// impl ExtBuilder {
// 	pub fn build(self) -> sp_io::TestExternalities {
// 		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
// 		let bob: AccountId32 = AccountId32::new([2u8; 32]);

// 		pallet_balances::GenesisConfig::<Test> {
// 			balances: vec![(ALICE, 1000000000), (bob, 1000000000)],
// 		}
// 		.assimilate_storage(&mut t)
// 		.unwrap();


// 		let mut ext = sp_io::TestExternalities::new(t);
// 		ext.execute_with(|| System::set_block_number(1));
// 		ext

// 	}
// }
