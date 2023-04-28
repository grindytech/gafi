use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{BoundedVec, pallet_prelude::Member, Parameter};
use sp_core::hashing::blake2_512;
use pallet_nfts::Incrementable;

pub type Amount = u32;
pub type Level = u8;
pub type Metadata<S> = BoundedVec<u8, S>;

#[derive(Debug, PartialEq, Encode, Decode, Clone, Copy, Eq, MaxEncodedLen)]
pub struct CollectionId(pub [u8; 64]);

impl From<u32> for CollectionId {
	fn from(value: u32) -> Self {
		CollectionId(value.using_encoded(blake2_512))
	}
}

impl CollectionId {
    fn default() -> Self {
        CollectionId([0; 64])
    }

    fn random(&self) -> Self {
        CollectionId([0; 64])
    }
}

impl Incrementable for CollectionId {
    fn increment(&self) -> Self {
        Self::random(self)
    }

    fn initial_value() -> Self {
        CollectionId::default()
    }
}

// <(<T as frame_system::Config>::Hash, <T as frame_system::Config>::BlockNumber)>
impl<T, U> From<(T, U)> for CollectionId
where
	T: Encode,
    U: Encode,
{
	fn from(value: (T, U)) -> Self {
		CollectionId(value.using_encoded(blake2_512))
	}
}
