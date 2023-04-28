use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::{
	game::{Support, CollectionId},
};
use sp_core::{hashing::blake2_512};

impl<T: Config<I>, I: 'static> Support for Pallet<T, I> {
	fn gen_id() -> CollectionId {
		let payload = (
			T::Randomness::random(&b""[..]).0,
			<frame_system::Pallet<T>>::block_number(),
		);
		CollectionId::from(payload)
	}
}
