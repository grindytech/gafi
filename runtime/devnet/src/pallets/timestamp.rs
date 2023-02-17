use frame_support::parameter_types;
use frame_support::traits::OnTimestampSet;

use crate::{Aura, Runtime, SLOT_DURATION, PhantomData};

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
	pub storage EnableManualSeal: bool = false;
}

pub struct ConsensusOnTimestampSet<T>(PhantomData<T>);
impl<T: pallet_aura::Config> OnTimestampSet<T::Moment> for ConsensusOnTimestampSet<T> {
	fn on_timestamp_set(moment: T::Moment) {
		if EnableManualSeal::get() {
			return;
		}
		<pallet_aura::Pallet<T> as OnTimestampSet<T::Moment>>::on_timestamp_set(moment)
	}
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ConsensusOnTimestampSet<Self>;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}
