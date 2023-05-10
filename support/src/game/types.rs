use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{BoundedVec};
use pallet_nfts::Incrementable;
use sp_io::hashing::blake2_256;

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
}

impl Incrementable for CollectionId {
	fn increment(&self) -> Self {
		*self
	}

	fn initial_value() -> Self {
		CollectionId::default()
	}
}

impl<Hash, BlockNumber> From<(Hash, BlockNumber)> for CollectionId
where
	Hash: Encode,
	BlockNumber: Encode,
{
	fn from(value: (Hash, BlockNumber)) -> Self {
		CollectionId(value.using_encoded(blake2_256))
	}
}
