use frame_support::parameter_types;
use gafi_tx::{GafiEVMCurrencyAdapter, GafiGasWeightMapping};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
// use runtime_common::impls::DealWithFees;
use sp_core::U256;

use crate::{
	precompiles::FrontierPrecompiles, AccountId, Aura, Balances, FindAuthorTruncated,
	ProofAddressMapping, Runtime, RuntimeEvent, TxHandler, Weight, NORMAL_DISPATCH_RATIO,
};

use crate::types::config::{BLOCK_GAS_LIMIT, WEIGHT_MILLISECS_PER_BLOCK};
use frame_support::weights::constants::WEIGHT_REF_TIME_PER_MILLIS;
use sp_runtime::Perbill;

/// `WeightPerGas` is an approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
///
/// `GAS_PER_MILLIS * WEIGHT_MILLIS_PER_BLOCK * TXN_RATIO ~= BLOCK_GAS_LIMIT`
/// `WEIGHT_PER_GAS = WEIGHT_REF_TIME_PER_MILLIS / GAS_PER_MILLIS
///                 = WEIGHT_REF_TIME_PER_MILLIS / (BLOCK_GAS_LIMIT / TXN_RATIO /
/// WEIGHT_MILLIS_PER_BLOCK)                 = TXN_RATIO * (WEIGHT_REF_TIME_PER_MILLIS *
/// WEIGHT_MILLIS_PER_BLOCK) / BLOCK_GAS_LIMIT`
///
/// For example, given the 2000ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is `GAS_PER_MILLIS * 2000 * 75% = BLOCK_GAS_LIMIT`.
pub fn weight_per_gas(
	block_gas_limit: u64,
	txn_ratio: Perbill,
	weight_millis_per_block: u64,
) -> u64 {
	let weight_per_block = WEIGHT_REF_TIME_PER_MILLIS.saturating_mul(weight_millis_per_block);
	let weight_per_gas = (txn_ratio * weight_per_block).saturating_div(block_gas_limit);
	assert!(
		weight_per_gas >= 1,
		"WeightPerGas must greater than or equal with 1"
	);
	weight_per_gas
}

parameter_types! {
	pub const ChainId: u64 = 1337;
	pub BlockGasLimit: U256 = U256::from(u32::max_value());
	pub PrecompilesValue: FrontierPrecompiles<Runtime> = FrontierPrecompiles::<_>::new();
	pub WeightPerGas: Weight = Weight::from_ref_time(weight_per_gas(BLOCK_GAS_LIMIT,
		NORMAL_DISPATCH_RATIO, WEIGHT_MILLISECS_PER_BLOCK));
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
	type WeightPerGas = WeightPerGas;
	type OnCreate = ();
}
