/*
 * This unittest should only test logic function e.g. Storage, Computation
 * and not related with Currency e.g. Balances, Transaction Payment
 */

use crate::{self as funding_pool};
use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{ConstU32},
};
use frame_system as system;

use frame_support::{
	dispatch::Vec,
	traits::{OnFinalize, OnInitialize},
};
use gafi_primitives::currency::{unit, NativeToken::GAKI};
pub use pallet_balances::Call as BalancesCall;
use sp_core::{H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, Permill,
};
use system::EnsureRoot;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const TIME_SERVICE: u128 = 60 * 60_000u128; // 1 hour

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
		Funding: funding_pool::{Pallet, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		PalletNick: pallet_nicks,
	}
);

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

parameter_types! {
	pub MinPoolBalance: u128 = 1000 * unit(GAKI);
	pub MinDiscountPercent: Permill = Permill::from_percent(10);
	pub MaxDiscountPercent: Permill = Permill::from_percent(70);
	pub MinTxLimit: u32 = 10;
	pub MaxTxLimit: u32 = 100;
	pub MaxPoolOwned: u32 =  10;
	pub MaxPoolTarget: u32 =  10;
}


pub const RESERVATION_FEE: u128 = 2;

ord_parameter_types! {
	pub const ReservationFee: u128 = RESERVATION_FEE * unit(GAKI);
	pub const One: AccountId32 = AccountId32::from([1; 32]);
}

impl pallet_nicks::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ReservationFee = ReservationFee;
	type Slashed = ();
	type ForceOrigin = EnsureRoot<AccountId32>;
	type MinLength = ConstU32<3>;
	type MaxLength = ConstU32<16>;
}

impl funding_pool::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type MaxPoolOwned = MaxPoolOwned;
	type MaxPoolTarget = MaxPoolTarget;
	type MinDiscountPercent = MinDiscountPercent;
	type MaxDiscountPercent = MaxDiscountPercent;
	type MinTxLimit = MinTxLimit;
	type MaxTxLimit = MaxTxLimit;
	type MinPoolBalance = MinPoolBalance;
	type IWhitelist = ();
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
		Timestamp::set_timestamp(
			(System::block_number() as u64 * MILLISECS_PER_BLOCK) + INIT_TIMESTAMP,
		);
	}
}

pub struct ExtBuilder {
	balances: Vec<(AccountId32, u128)>,
	pub time_service: u128,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![],
			time_service: TIME_SERVICE,
		}
	}
}

impl ExtBuilder {
	fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let _ = pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut storage);

		let ext = sp_io::TestExternalities::from(storage);
		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let mut ext = self.build();
		ext.execute_with(test);
		ext.execute_with(|| System::set_block_number(1));
	}
}
