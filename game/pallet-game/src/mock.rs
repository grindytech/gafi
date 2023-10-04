use crate::{self as pallet_game};
use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU64, OnFinalize, OnInitialize},
	PalletId,
};
use frame_system as system;
use pallet_balances::AccountData;
use pallet_nfts::PalletFeatures;
use sp_core::{sr25519::Signature, ConstU128, ConstU32, H256};
use sp_runtime::{
	testing::TestXt,
	traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
};
use system::mocking;

pub type Extrinsic = TestXt<RuntimeCall, ()>;
type Block = mocking::MockBlock<Test>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
type AccountPublic = <Signature as Verify>::Signer;
use sp_runtime::BuildStorage;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		PalletGame: pallet_game,
		Balances: pallet_balances,
		Nfts: pallet_nfts,
	}
);

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
	type AccountId = sp_core::sr25519::Public;
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
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

pub const EXISTENTIAL_DEPOSIT: u128 = 1000;

parameter_types! {
	pub ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u128;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}

pub const ITEM_DEPOSIT_VAL: u128 = 3_000_000_000;
pub const METADATA_DEPOSIT_VAL: u128 = 3_000_000_000;
pub const BYTE_DEPOSIT_VAL: u128 = 3_000_000;

parameter_types! {
	pub storage Features: PalletFeatures = PalletFeatures::all_enabled();
	pub ItemDeposit: u128 = ITEM_DEPOSIT_VAL;
	pub MetadataDepositBase: u128 = METADATA_DEPOSIT_VAL;
	pub DepositPerByte: u128 = BYTE_DEPOSIT_VAL;
}

impl pallet_nfts::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<Self::AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type Locker = ();
	type CollectionDeposit = ConstU128<2>;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type AttributeDepositBase = ConstU128<1>;
	type DepositPerByte = DepositPerByte;
	type StringLimit = ConstU32<50>;
	type KeyLimit = ConstU32<50>;
	type ValueLimit = ConstU32<50>;
	type ApprovalsLimit = ConstU32<10>;
	type ItemAttributesApprovalsLimit = ConstU32<2>;
	type MaxTips = ConstU32<10>;
	type MaxDeadlineDuration = ConstU64<10000>;
	type MaxAttributesPerCall = ConstU32<2>;
	type Features = Features;
	/// Off-chain = signature On-chain - therefore no conversion needed.
	/// It needs to be From<MultiSignature> for benchmarking.
	type OffchainSignature = Signature;
	/// Using `AccountPublic` here makes it trivial to convert to `AccountId` via `into_account()`.
	type OffchainPublic = AccountPublic;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
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
	RuntimeCall: From<C>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		_public: <Signature as Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(RuntimeCall, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

pub const GAME_DEPOSIT_VAL: u128 = 5_000_000_000;
pub const UPGRADE_DEPOSIT_VAL: u128 = 3_000_000_000;
pub const BUNDLE_DEPOSIT_VAL: u128 = 3_000_000_000;
pub const MINING_DEPOSIT_VAL: u128 = 5_000_000_000;
pub const MAX_GAME_COLLECTION_VAL: u32 = 10;
pub const MAX_ITEM_MINT_VAL: u32 = 10;
pub const MAX_GAME_SHARE_VAL: u32 = 10;
pub const MAX_BUNDLE_VAL: u32 = 5;
pub const MAX_LOOT: u32 = 10;
pub const MAX_MIN_REQUEST_VAL: u32 = 10;
pub const MIN_INTERVAL_VAL: u32 = 1;

parameter_types! {
	pub GameDeposit: u128 = GAME_DEPOSIT_VAL;
	pub MaxGameCollection: u32 = MAX_GAME_COLLECTION_VAL;
	pub MiningPoolDeposit: u128 = MINING_DEPOSIT_VAL;
	pub UpgradeDeposit: u128 = UPGRADE_DEPOSIT_VAL;
	pub BundleDeposit: u128 = BUNDLE_DEPOSIT_VAL;

	pub MaxMintItem: u32 = MAX_ITEM_MINT_VAL;
	pub MaxBundle: u32 = MAX_BUNDLE_VAL;
	pub MaxGameShare: u32 = MAX_GAME_SHARE_VAL;

	pub PalletGameId: PalletId =  PalletId(*b"gamegame");
	pub MaxItem: u32 = 10;
	pub MaxLoot: u32 = MAX_LOOT;
	pub MaxMintRequest: u32 = MAX_MIN_REQUEST_VAL;
	pub MintInterval: u32 = MIN_INTERVAL_VAL;
}

impl pallet_game::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type NftsWeightInfo = ();
	type Currency = Balances;
	type Nfts = Nfts;
	type GameId = u32;
	type TradeId = u32;
	type PoolId = u32;
	type MiningPoolDeposit = MiningPoolDeposit;
	type GameDeposit = GameDeposit;
	type MaxGameCollection = MaxGameCollection;
	type MaxGameShare = MaxGameShare;
	type MaxItem = MaxItem;
	type MaxMintItem = MaxMintItem;
	type UpgradeDeposit = UpgradeDeposit;
	type BundleDeposit = BundleDeposit;
	type MaxBundle = MaxBundle;
	type MaxLoot = MaxLoot;
	type GameRandomness = ();
	type MaxMintRequest = MaxMintRequest;
	type MintInterval = MintInterval;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
			PalletGame::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		PalletGame::on_initialize(System::block_number());
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
