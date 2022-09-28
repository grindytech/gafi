use frame_support::parameter_types;

use crate::{Aura, Runtime, SLOT_DURATION};

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
	#[cfg(all(feature = "aura", not(feature = "manual-seal")))]
	type OnTimestampSet = Aura;
	#[cfg(feature = "manual-seal")]
	type OnTimestampSet = ();
}
