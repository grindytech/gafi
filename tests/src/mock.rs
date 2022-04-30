use std::str::FromStr;

use frame_support::{
	dispatch::Vec,
	traits::{Currency, OnFinalize, OnInitialize},
};
use frame_support::{
	parameter_types,
	traits::{ConstU8, GenesisBuild},
	weights::IdentityFee,
};
use frame_system as system;
use gafi_primitives::currency::{centi, unit, NativeToken::GAKI};
use gafi_primitives::pool::{FlexService, Level, Service, TicketType};
use gafi_tx::GafiEVMCurrencyAdapter;
use hex_literal::hex;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use pallet_timestamp;
use pallet_transaction_payment::CurrencyAdapter;
use proof_address_mapping::ProofAddressMapping;
use sp_core::{H160, H256, U256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};

pub use pallet_balances::Call as BalancesCall;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

fn get_accountid32(addr: &str) -> AccountId32 {
	AccountId32::from_str(addr).unwrap()
}

pub const PREFIX: &[u8] = b"Bond Gafi Network account:";

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
		UpfrontPool: upfront_pool::{Pallet, Call, Storage, Event<T>},
		Pool: pallet_pool::{Pallet, Call, Storage, Event<T>},
		StakingPool: staking_pool::{Pallet, Storage, Event<T>},
		SponsoredPool: sponsored_pool::{Pallet, Storage, Event<T>},
		PalletTxHandler: gafi_tx::{Pallet, Call, Storage, Event<T>},
		PalletAddressMapping: proof_address_mapping::{Pallet, Call, Storage, Event<T>},
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, Origin},
		EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
	}
);

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub Prefix: &'static [u8] =  PREFIX;
}

impl proof_address_mapping::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
	type MessagePrefix = Prefix;
}

impl pallet_transaction_payment::Config for Test {
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = IdentityFee<u128>;
	type LengthToFee = IdentityFee<u128>;
	type FeeMultiplierUpdate = ();
}

parameter_types! {
	pub const ChainId: u64 = 1337;
	pub BlockGasLimit: U256 = U256::from(u32::max_value());
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressRoot<AccountId32>;
	type WithdrawOrigin = EnsureAddressNever<AccountId32>;
	type AddressMapping = ProofAddressMapping<Self>;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type OnChargeTransaction = GafiEVMCurrencyAdapter<Balances, ()>;
	type FindAuthor = ();
}

impl pallet_ethereum::Config for Test {
	type Event = Event;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}

impl pallet_pool::Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type Currency = Balances;
	type UpfrontPool = UpfrontPool;
	type StakingPool = StakingPool;
	type SponsoredPool = SponsoredPool;
}

parameter_types! {
	pub MaxPlayerStorage: u32 = 1000;
}

impl upfront_pool::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
	type MaxPlayerStorage = MaxPlayerStorage;
	type MasterPool = Pool;
}

impl staking_pool::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
}

parameter_types! {
	pub MaxPoolOwned: u32 =  10;
	pub MaxPoolTarget: u32 = 10;
}

impl sponsored_pool::Config for Test {
	type Event = Event;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type MaxPoolOwned = MaxPoolOwned;
	type MaxPoolTarget = MaxPoolTarget;
	type WeightInfo = ();
}

pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = 6 * MILLISECS_PER_BLOCK; // 6 seconds
pub const TIME_SERVICE: u128 = 60 * 60_000u128; // 1 hour

pub const INIT_TIMESTAMP: u64 = 0;

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
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

impl gafi_tx::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type OnChargeEVMTxHandler = ();
	type AddressMapping = ProofAddressMapping<Self>;
	type PlayerTicket = Pool;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			UpfrontPool::on_finalize(System::block_number());
			Pool::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		UpfrontPool::on_initialize(System::block_number());
		Pool::on_initialize(System::block_number());
		Timestamp::set_timestamp((System::block_number() as u64 * SLOT_DURATION) + INIT_TIMESTAMP);
	}
}

pub struct ExtBuilder {
	balances: Vec<(AccountId32, u128)>,
	pub max_player: u32,
	pub time_service: u128,
	pub upfront_services: [(Level, FlexService); 3],
	pub staking_services: [(Level, FlexService); 3],
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![],
			max_player: 1000,
			time_service: TIME_SERVICE,
			upfront_services: [
				(
					Level::Basic,
					FlexService::new(100_u32, 30_u8, 5 * unit(GAKI)),
				),
				(
					Level::Medium,
					FlexService::new(100_u32, 50_u8, 7 * unit(GAKI)),
				),
				(
					Level::Advance,
					FlexService::new(100_u32, 70_u8, 10 * unit(GAKI)),
				),
			],
			staking_services: [
				(
					Level::Basic,
					FlexService::new(100_u32, 30_u8, 1000 * unit(GAKI)),
				),
				(
					Level::Medium,
					FlexService::new(100_u32, 50_u8, 1500 * unit(GAKI)),
				),
				(
					Level::Advance,
					FlexService::new(100_u32, 70_u8, 2000 * unit(GAKI)),
				),
			],
		}
	}
}

impl ExtBuilder {
	fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		let _ = pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut storage);

		GenesisBuild::<Test>::assimilate_storage(
			&upfront_pool::GenesisConfig {
				max_player: self.max_player,
				services: self.upfront_services,
			},
			&mut storage,
		)
		.unwrap();
		GenesisBuild::<Test>::assimilate_storage(
			&staking_pool::GenesisConfig {
				services: self.staking_services,
			},
			&mut storage,
		)
		.unwrap();
		GenesisBuild::<Test>::assimilate_storage(
			&pallet_pool::GenesisConfig {
				time_service: self.time_service,
			},
			&mut storage,
		)
		.unwrap();

		let mut ext = sp_io::TestExternalities::from(storage);
		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let mut ext = self.build();
		ext.execute_with(test);
		ext.execute_with(|| System::set_block_number(1));
	}
}
