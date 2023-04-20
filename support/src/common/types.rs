use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiSignature,
};

pub type BlockNumber = u32;

pub type Signature = MultiSignature;

pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

type AccountPublic = <Signature as Verify>::Signer;

pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

pub type Block = generic::Block<Header, UncheckedExtrinsic>;

pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

pub type Balance = u128;

pub type Hash = sp_core::H256;

pub type Index = u32;
