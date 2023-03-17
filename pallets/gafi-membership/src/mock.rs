use crate::{self as gafi_membership, UpfrontPoolTimeAchievement};
use frame_support::{
	parameter_types,
	traits::{GenesisBuild, Get, OnFinalize, OnInitialize},
	BoundedVec,
};
use frame_system as system;
use gafi_primitives::membership::{Achievements, MembershipLevelPoints};
pub use gu_mock::{one_mil_gaki, pool::*, INIT_TIMESTAMP, MILLISECS_PER_BLOCK, SLOT_DURATION};
use sp_core::H256;
pub use sp_runtime::AccountId32;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

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
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		GafiMembership: gafi_membership::{Pallet, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		Player: pallet_player::{Pallet, Call, Storage, Event<T>},
		UpfrontPool: upfront_pool::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type AccountData = pallet_balances::AccountData<u128>;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
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

impl pallet_randomness_collective_flip::Config for Test {}

pub const EXISTENTIAL_DEPOSIT: u128 = 1000;

parameter_types! {
	pub ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u128;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

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
	pub MaxPlayerStorage: u32 = 1000;
}

impl upfront_pool::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = ();
	type MaxPlayerStorage = MaxPlayerStorage;
	type MasterPool = ();
	type UpfrontServices = UpfrontPoolDefaultServices;
}

impl pallet_player::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type GameRandomness = RandomnessCollectiveFlip;
	type UpfrontPool = UpfrontPool;
	type StakingPool = ();
}

parameter_types! {
	pub const MaxMembers: u32 = 5u32;
	pub const MinJoinTime: u128 = 60 * 60_000u128; // 60 minutes
	pub const MaxAchievement: u32 = 100;
	pub const TotalMembershipLevel: u32 = 10;
}

pub struct MembershipAchievements {}

impl Achievements<UpfrontPoolTimeAchievement<Test, Player>, MaxAchievement>
	for MembershipAchievements
{
	fn get_membership_achievements(
	) -> BoundedVec<UpfrontPoolTimeAchievement<Test, Player>, MaxAchievement> {
		vec![UpfrontPoolTimeAchievement {
			phantom: Default::default(),
			id: [20; 32],
			min_joined_time: MinJoinTime::get(),
		}]
		.try_into()
		.unwrap_or_default()
	}
}

pub struct MembershipLevels {}

impl<TotalMembershipLevel: Get<u32>> MembershipLevelPoints<TotalMembershipLevel>
	for MembershipLevels
{
	fn get_membership_level_points() -> BoundedVec<u32, TotalMembershipLevel> {
		vec![50, 100, 200, 400].try_into().unwrap_or_default()
	}
}

impl gafi_membership::Config for Test {
	type Currency = Balances;
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type ApproveOrigin = system::EnsureRoot<AccountId32>;
	type MinJoinTime = MinJoinTime;
	type MaxMembers = MaxMembers;
	type Players = Player;
	type MaxAchievement = MaxAchievement;
	type Achievements = MembershipAchievements;
	type TotalMembershipLevel = TotalMembershipLevel;
	type MembershipLevelPoints = MembershipLevels;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

	GenesisBuild::<Test>::assimilate_storage(&upfront_pool::GenesisConfig::default(), &mut storage)
		.unwrap();

	let ext = sp_io::TestExternalities::from(storage);
	ext
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			GafiMembership::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Timestamp::set_timestamp(
			(System::block_number() as u64 * MILLISECS_PER_BLOCK) + INIT_TIMESTAMP,
		);
	}
}
