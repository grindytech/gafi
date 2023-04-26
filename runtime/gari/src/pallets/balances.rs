use crate::{Balance, Runtime, System, RuntimeEvent};
use frame_support::parameter_types;
use gafi_support::common::{unit, NativeToken::GAFI};

parameter_types! {
	pub  NativeTokenExistentialDeposit: Balance = 1 * unit(GAFI); // 1 GAFI
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = NativeTokenExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}
