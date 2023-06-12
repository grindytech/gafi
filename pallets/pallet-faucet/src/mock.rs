use crate as pallet_faucet;
use frame_support::parameter_types;
use frame_system as system;

use frame_support::{
	dispatch::Vec,
};
use sp_core::{H256, ConstU32, ConstU128, ConstU64};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
pub use pallet_balances::Call as BalancesCall;

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
		Faucet: pallet_faucet::{Pallet, Call, Storage, Event<T>},
		PalletCache: pallet_cache::{Pallet, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
	}
);

pub const EXISTENTIAL_DEPOSIT: u64 = 1000;

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = u64;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type HoldIdentifier = ();
	type MaxHolds = ();
}

pub const FAUCET_BALANCE: u64 = 1_000_000;
pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = 6 * MILLISECS_PER_BLOCK; // 6 seconds
pub const TIME_SERVICE: u128 = 60 * 60_000u128; // 1 hour

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
	pub CleanTime: u128 = TIME_SERVICE;
}

impl pallet_cache::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Action = AccountId32;
	type Data = u128;
	type CleanTime = CleanTime;
}

parameter_types! {
	pub FaucetAmount: u64 = FAUCET_BALANCE;
}

impl pallet_faucet::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = ();
	type Cache = PalletCache;
	type FaucetAmount = FaucetAmount;
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
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// Build genesis storage according to the mock runtime.
pub fn _new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub const GENESIS_ACCOUNT: AccountId32 = AccountId32::new([0u8; 32]);

pub const TEST_ACCOUNTS: [(AccountId32, u64); 10] = [
	(GENESIS_ACCOUNT, 1000000000000000000),
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

pub struct ExtBuilder {
	balances: Vec<(AccountId32, u64)>,
	genesis_accounts: Vec<AccountId32>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: TEST_ACCOUNTS.to_vec(),
			genesis_accounts: vec![
				GENESIS_ACCOUNT,
				TEST_ACCOUNTS[1].0.clone(),
				TEST_ACCOUNTS[2].0.clone(),
				TEST_ACCOUNTS[3].0.clone(),
				TEST_ACCOUNTS[4].0.clone(),
				TEST_ACCOUNTS[5].0.clone(),
			],
		}
	}
}

impl ExtBuilder {
	fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let _ = pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut storage);

		let _ = pallet_faucet::GenesisConfig::<Test> { genesis_accounts: self.genesis_accounts }
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
