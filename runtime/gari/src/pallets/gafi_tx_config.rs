use frame_support::parameter_types;
use sp_runtime::Permill;
use crate::{Balances, Runtime, ProofAddressMapping, Pool, RuntimeEvent};
use pallet_evm::EVMCurrencyAdapter;

parameter_types! {
	pub GameCreatorReward: Permill = Permill::from_percent(30_u32);
	pub GasPrice: u128 = 4_000_000_000_u128;
}

impl gafi_tx::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type OnChargeEVMTxHandler = EVMCurrencyAdapter<Balances, ()>;
	type AddressMapping = ProofAddressMapping;
	type PlayerTicket = Pool;
	type GameCreatorReward = GameCreatorReward;
	type GetGameCreator = ();
	type GasPrice = GasPrice;
}