use crate as proof_address_mapping;
use frame_support::parameter_types;
use frame_system as system;

use frame_support::traits::{OnFinalize, OnInitialize};
use sp_core::{H256, U256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressTruncated, EVMCurrencyAdapter};
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
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, Origin},
		EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		ProofAddressMapping: proof_address_mapping::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const ChainId: u64 = 1337;
	pub BlockGasLimit: U256 = U256::from(u32::max_value());
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressTruncated;
	type WithdrawOrigin = EnsureAddressNever<AccountId32>;
	type AddressMapping = ProofAddressMapping;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type OnChargeTransaction = EVMCurrencyAdapter<Balances, ()>;
	type FindAuthor = ();
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

impl pallet_ethereum::Config for Test {
	type Event = Event;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}

parameter_types! {
	pub Prefix: &'static [u8] =  b"Bond Aurora Network account:";
	pub ReservationFee: u64 = RESERVATION_FEE;
}

impl proof_address_mapping::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
	type MessagePrefix = Prefix;
	type ReservationFee = ReservationFee;
}

pub const EXISTENTIAL_DEPOSIT: u64 = 1000;
pub const RESERVATION_FEE: u64 = 1000;

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


// Build genesis storage according to the mock runtime.
pub fn _new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
	}
}

pub struct ExtBuilder {
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
		}
	}
}

impl ExtBuilder {
	fn build(self) -> sp_io::TestExternalities {
		let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let ext = sp_io::TestExternalities::from(storage);
		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let mut ext = self.build();
		ext.execute_with(test);
		ext.execute_with(|| System::set_block_number(1));
	}
}