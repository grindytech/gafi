/*
 * This unittest should only test logic function e.g. Storage, Computation
 * and not related with Currency e.g. Balances, Transaction Payment
 */

use crate::{self as pallet_whitelist};
use frame_support::{
	parameter_types,
	traits::{ConstU32, GenesisBuild},
};
use frame_system as system;

use frame_support::{
	dispatch::Vec,
	traits::{OnFinalize, OnInitialize},
};
use gafi_primitives::{
	constant::ID,
	currency::{unit, NativeToken::GAKI},
	ticket::TicketInfo,
};
pub use pallet_balances::Call as BalancesCall;

use frame_system::{mocking};
use sp_core::{
	sr25519::{self, Signature},
	H256,
};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{
		BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup,
		Verify,
	}, Permill,
};

pub type Extrinsic = TestXt<Call, ()>;
type UncheckedExtrinsic = mocking::MockUncheckedExtrinsic<Test>;
type Block = mocking::MockBlock<Test>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

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
		PalletWhitelist: pallet_whitelist::{Pallet, Call, Storage, Event<T>},
	}
);

pub const EXISTENTIAL_DEPOSIT: u128 = 1000;

parameter_types! {
	pub ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT;
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

impl  pallet_whitelist::Config for Test {
	type Event = Event;
	type WhitelistPool = ();
	type WhitelistSponsor = ();
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
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = sr25519::Public;
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

parameter_types! {
	pub const UnsignedPriority: u64 = 100;
}

impl frame_system::offchain::SigningTypes for Test {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Test
where
	Call: From<C>,
{
	type OverarchingCall = Call;
	type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		_public: <Signature as Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

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
	}
}

pub struct ExtBuilder {
	balances: Vec<(sr25519::Public, u128)>,
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

		let ext = sp_io::TestExternalities::from(storage);
		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let mut ext = self.build();
		ext.execute_with(test);
		ext.execute_with(|| System::set_block_number(1));
	}
}
