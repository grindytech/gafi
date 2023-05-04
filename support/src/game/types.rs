use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{BoundedVec, pallet_prelude::Member, Parameter};
use pallet_nfts::Incrementable;
use sp_core::{blake2_256};

pub type Amount = u32;
pub type Level = u8;
pub type Metadata<S> = BoundedVec<u8, S>;

#[derive(Debug, PartialEq, Encode, Decode, Clone, Copy, Eq, MaxEncodedLen)]
pub struct CollectionId(pub [u8; 32]);

impl From<u32> for CollectionId {
	fn from(value: u32) -> Self {
		CollectionId(value.using_encoded(blake2_256))
	}
}

impl CollectionId {
    fn default() -> Self {
        CollectionId([0; 32])
    }

    fn random(&self) -> Self {
        CollectionId([0; 32])
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
		CollectionId(value.using_encoded(blake2_256))
	}
}
