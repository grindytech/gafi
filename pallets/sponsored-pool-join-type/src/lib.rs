#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use gafi_primitives::{
		constant::ID,
		pool::{SponsoredPoolJoinType, SponsoredPoolJoinTypeHandle},
	};
	use sp_std::vec::Vec;
	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The maximum length a url.
		#[pallet::constant]
		type MaxLength: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn external_of)]
	pub(super) type ExternalOf<T: Config> =
		StorageMap<_, Twox64Concat, ID, (SponsoredPoolJoinType, BoundedVec<u8, T::MaxLength>)>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ChangeJoinType(ID, SponsoredPoolJoinType, Vec<u8>, T::AccountId),
		ResetJoinType(ID, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		UrlTooLong,
		UrlInvalid,
		NotFound,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T: Config> SponsoredPoolJoinTypeHandle<AccountIdOf<T>> for Pallet<T> {
		fn set_join_type(
			pool_id: ID,
			join_type: SponsoredPoolJoinType,
			call_check_url: Vec<u8>,
			account_id: AccountIdOf<T>,
		) -> DispatchResult {
			let bounded_url: BoundedVec<u8, T::MaxLength> =
				call_check_url.clone().try_into().map_err(|()| Error::<T>::UrlTooLong)?;
			<ExternalOf<T>>::insert(pool_id, (join_type, bounded_url));
			Self::deposit_event(Event::<T>::ChangeJoinType(
				pool_id,
				join_type,
				call_check_url,
				account_id,
			));
			Ok(())
		}
		fn reset(pool_id: ID, account_id: AccountIdOf<T>) -> DispatchResult {
			<ExternalOf<T>>::remove(pool_id);
			Self::deposit_event(Event::<T>::ResetJoinType(pool_id, account_id));
			Ok(())
		}
		fn get_join_type(pool_id: ID) -> Option<(SponsoredPoolJoinType, Vec<u8>)> {
			match <ExternalOf<T>>::get(pool_id).ok_or(Error::<T>::NotFound) {
				Ok((join_type, bounded_url)) =>
					return Some((join_type, Vec::<u8>::from(bounded_url))),
				Err(_) => return None,
			}
		}
	}
}
