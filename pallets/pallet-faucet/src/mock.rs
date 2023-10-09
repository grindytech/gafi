use crate as pallet_faucet;
use frame_support::parameter_types;
use frame_system as system;

use frame_support::dispatch::Vec;
use pallet_balances::AccountData;
pub use pallet_balances::Call as BalancesCall;
use sp_core::{ConstU128, ConstU64, H256, ConstU16, ConstU32};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
use sp_runtime::BuildStorage;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Faucet: pallet_faucet,
		PalletCache: pallet_cache,
		Timestamp: pallet_timestamp,
	}
);

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u128;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}

pub const FAUCET_BALANCE: u128 = 1_000_000;
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
	pub FaucetAmount: u128 = FAUCET_BALANCE;
}

impl pallet_faucet::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = ();
	type Cache = PalletCache;
	type FaucetAmount = FaucetAmount;
	type MaxFundingAccount = ConstU32<5>;
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
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// Build genesis storage according to the mock runtime.
pub fn _new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

pub const GENESIS_ACCOUNT: AccountId32 = AccountId32::new([0u8; 32]);

pub const TEST_ACCOUNTS: [(AccountId32, u128); 10] = [
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
	balances: Vec<(AccountId32, u128)>,
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
		let mut storage =
			frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();

		let _ = pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut storage);

		let _ = pallet_faucet::GenesisConfig::<Test> {
			genesis_accounts: self.genesis_accounts,
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
