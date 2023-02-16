use frame_support::parameter_types;
use gafi_tx::{GafiEVMCurrencyAdapter, GafiGasWeightMapping};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
// use runtime_common::impls::DealWithFees;
use sp_core::U256;

use crate::{
	precompiles::FrontierPrecompiles, AccountId, Aura, Balances, Event, FindAuthorTruncated,
	ProofAddressMapping, Runtime, TxHandler,
};

parameter_types! {
	pub const ChainId: u64 = 1337;
	pub BlockGasLimit: U256 = U256::from(u32::max_value());
	pub PrecompilesValue: FrontierPrecompiles<Runtime> = FrontierPrecompiles::<_>::new();
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = TxHandler;
	type GasWeightMapping = GafiGasWeightMapping;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = ProofAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = FrontierPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type OnChargeTransaction = GafiEVMCurrencyAdapter<Balances, ()>;
	type FindAuthor = FindAuthorTruncated<Aura>;
}
