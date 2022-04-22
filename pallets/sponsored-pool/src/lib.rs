#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::{pallet_prelude::*};
use frame_system::pallet_prelude::*;
pub use gafi_primitives::{pool::{Level, Service, StaticPool}, constant::ID};
use frame_support::traits::{Randomness};
use sp_io::hashing::blake2_256;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};


#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct SponsoredPool {
	pub id: ID,
	pub value: u128,
	pub discount: u8,
	pub tx_limit: u32,
}

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;


	#[pallet::config]
	pub trait Config: frame_system::Config {

		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	//** Storages **//
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Pools<T: Config> = StorageMap<_, Twox64Concat, ID, SponsoredPool>;

	#[pallet::storage]
	pub(super) type PoolOwned<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, ID>;

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

			let pool_id = Self::gen_id()?;



			let new_pool = SponsoredPool {
				id: pool_id,
				value,
				discount,
				tx_limit,
			};

			PoolOwned::<T>::insert(sender.clone(), pool_id);
			Pools::<T>::insert(pool_id, new_pool);

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
			if let Some(pool) = Pools::<T>::get(pool_id) {
				return Some(Service {
					discount: pool.discount,
					value: 0u128,
					tx_limit: pool.tx_limit,
				})
			}
			None
		}
	}
}
