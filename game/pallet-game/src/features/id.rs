use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::{
	game::Support,
};
use sp_core::blake2_256;

impl<T: Config<I>, I: 'static> Support for Pallet<T, I> {
	fn gen_id() -> Option<ID> {
		let payload = (
			T::Randomness::random(&b""[..]).0,
			<frame_system::Pallet<T>>::block_number(),
		);
		Some(payload.using_encoded(blake2_256))
	}
}
