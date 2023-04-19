use frame_support::{
	dispatch::Vec,
	parameter_types,
	traits::{ConstU32, ConstU8, GenesisBuild, OnFinalize, OnInitialize},
	weights::{IdentityFee, Weight}, ord_parameter_types,
};
use frame_system as system;
use gafi_primitives::{
	common::{constant::ID,
	currency::{unit, NativeToken::GAKI}},
	pool::ticket::TicketInfo,
};
pub use gu_mock::{pool::*, one_mil_gaki};
pub use pallet_balances::Call as BalancesCall;
use pallet_evm::{EnsureAddressNever, EnsureAddressTruncated, EVMCurrencyAdapter, GasWeightMapping};
use pallet_timestamp;
use pallet_transaction_payment::CurrencyAdapter;
use sp_core::{H256, U256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, Permill,
};
use system::EnsureRoot;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

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
		FundingPool: funding_pool::{Pallet, Storage, Event<T>},
		PalletCache: pallet_cache::{Pallet, Storage, Event<T>},
		PalletTxHandler: gafi_tx::{Pallet, Call, Storage, Event<T>},
		ProofAddressMapping: proof_address_mapping::{Pallet, Call, Storage, Event<T>},
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, Origin},
		EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		GameCreator: game_creator::{Pallet, Call, Storage, Event<T>},
		Players: pallet_player::{Pallet, Call, Storage, Event<T>},
		PalletNicks: pallet_nicks,
	}
);

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub MaxContractOwned: u32 = 1000;
	pub GameCreatorFee: u128 = 5 * unit(GAKI);
}

impl game_creator::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AddressMapping = ProofAddressMapping;
	type MaxContractOwned = MaxContractOwned;
	type ContractCreator = EVM;
	type ReservationFee = GameCreatorFee;
	type WeightInfo = ();
}

parameter_types! {
	pub Prefix: &'static [u8] =  PREFIX;
	pub Fee: u128 = unit(GAKI);
}

impl proof_address_mapping::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = ();
	type MessagePrefix = Prefix;
	type ReservationFee = Fee;
}

impl pallet_transaction_payment::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = IdentityFee<u128>;
	type LengthToFee = IdentityFee<u128>;
	type FeeMultiplierUpdate = ();
}

pub struct TestGasWeightMapping;
impl GasWeightMapping for TestGasWeightMapping {
	fn gas_to_weight(gas: u64, _without_base_weight: bool) -> Weight {
		Weight::from_ref_time(gas)
	}

	fn weight_to_gas(weight: Weight) -> u64 {
		weight.ref_time()
	}
}

parameter_types! {
	pub const ChainId: u64 = 1337;
	pub BlockGasLimit: U256 = U256::from(u32::max_value());
	pub WeightPerGas: Weight = Weight::from_ref_time(0_u64);
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressTruncated;
	type WithdrawOrigin = EnsureAddressNever<AccountId32>;
	type AddressMapping = ProofAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type OnChargeTransaction = EVMCurrencyAdapter<Balances, ()>;
	type FindAuthor = ();
	type WeightPerGas = WeightPerGas;
	type OnCreate = ();
	type GasWeightMapping = TestGasWeightMapping;
}

impl pallet_ethereum::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}

parameter_types! {
	pub CleanTime: u128 = TIME_SERVICE;
}

impl pallet_cache::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Data = TicketInfo;
	type Action = ID;
	type CleanTime = CleanTime;
}

parameter_types! {
	pub MaxJoinedFundingPool: u32 = 5_u32;
	pub TimeServiceStorage: u128 = TIME_SERVICE;
}

impl pallet_pool::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type UpfrontPool = UpfrontPool;
	type StakingPool = StakingPool;
	type FundingPool = FundingPool;
	type MaxJoinedFundingPool = MaxJoinedFundingPool;
	type Cache = PalletCache;
	type TimeServiceStorage = TimeServiceStorage;
}

impl pallet_player::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type GameRandomness = RandomnessCollectiveFlip;
	type UpfrontPool = UpfrontPool;
	type StakingPool = StakingPool;
}

parameter_types! {
	pub MaxPlayerStorage: u32 = 1000;
}

impl upfront_pool::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = ();
	type MaxPlayerStorage = MaxPlayerStorage;
	type MasterPool = Pool;
	type UpfrontServices = UpfrontPoolDefaultServices;
}

impl staking_pool::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = ();
	type StakingServices = StakingPoolDefaultServices;
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

parameter_types! {
	pub MaxPoolOwned: u32 =  10;
	pub MaxPoolTarget: u32 = 10;
	pub MinPoolBalance: u128 = 1000 * unit(GAKI);
	pub MinDiscountPercent: Permill = Permill::from_percent(10);
	pub MaxDiscountPercent: Permill = Permill::from_percent(70);
	pub MinTxLimit: u32 = 10;
	pub MaxTxLimit: u32 = 100;
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
	type WeightInfo = ();
	type IWhitelist = ();
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
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
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
	pub GameCreatorReward: Permill = Permill::from_percent(30);
	pub GasPrice: u128 = 1_u128;
}

impl gafi_tx::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type OnChargeEVMTxHandler = ();
	type AddressMapping = ProofAddressMapping;
	type PlayerTicket = Pool;
	type GameCreatorReward = GameCreatorReward;
	type GetGameCreator = GameCreator;
	type GasPrice = GasPrice;
}

// Build genesis storage according to the mock runtime.
pub fn _new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
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
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balances: vec![] }
	}
}

impl ExtBuilder {
	fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let _ = pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut storage);

		GenesisBuild::<Test>::assimilate_storage(&upfront_pool::GenesisConfig {}, &mut storage)
			.unwrap();
		GenesisBuild::<Test>::assimilate_storage(&staking_pool::GenesisConfig {}, &mut storage)
			.unwrap();
		GenesisBuild::<Test>::assimilate_storage(&pallet_pool::GenesisConfig {}, &mut storage)
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
