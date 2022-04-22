#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(feature = "std")]
pub use gafi_primitives::{pool::{Level, Service, StaticPool}, constant::ID};
use frame_support::{serde::{Deserialize, Serialize},
	traits::Randomness,
};
use sp_io::hashing::blake2_256;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {

		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
	)]
	pub struct Airdrop {
		pub id: ID,
		pub value: u128,
		pub discount: u8,
		pub tx_limit: u32,
	}


	//** Storages **//
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Airdrops<T: Config> = StorageMap<_, Twox64Concat, ID, Airdrop>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored(u32, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn create_pool(origin: OriginFor<T>, value: u128, discount: u8, tx_limit: u32) -> DispatchResult {

			let sender = ensure_signed(origin)?;



			Ok(())
		}

		#[pallet::weight(0)]
		pub fn withdraw_pool(origin: OriginFor<T>) -> DispatchResult {
			
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn gen_id() -> Result<ID, Error<T>> {
			let payload =
				(T::Randomness::random(&b""[..]).0, <frame_system::Pallet<T>>::block_number());
			Ok(payload.using_encoded(blake2_256))
		}
	}

	impl<T: Config> StaticPool<T::AccountId> for Pallet<T> {
		fn join(sender: T::AccountId, pool_id: ID) -> DispatchResult {
			Ok(())
		}
	
		fn leave(sender: T::AccountId) -> DispatchResult {
			Ok(())
		}
		fn get_service(pool_id: ID) -> Option<Service> {
			if let Some(airdrop) = Airdrops::<T>::get(pool_id) {
				return Some(Service {
					discount: airdrop.discount,
					value: 0u128,
					tx_limit: airdrop.tx_limit,
				})
			}
			None
		}
	}
}
