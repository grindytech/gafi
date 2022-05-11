#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, OnUnbalanced, ReservableCurrency};

pub use pallet::*;
use sp_runtime::traits::Zero;
use sp_std::prelude::*;
pub use gafi_primitives::constant::ID;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Config>::Currency as Currency<AccountIdOf<T>>>::NegativeImbalance;

#[cfg(test)]
mod mocks;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Reservation fee.
		#[pallet::constant]
		type ReservationFee: Get<BalanceOf<Self>>;

		/// What to do with slashed funds.
		type Slashed: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// The minimum length a name may be.
		#[pallet::constant]
		type MinLength: Get<u32>;

		/// The maximum length a name may be.
		#[pallet::constant]
		type MaxLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A name was set.
		NameSet { pool: ID },
		/// A name was forcibly set.
		NameForced { target: ID },
		/// A name was changed.
		NameChanged { pool: ID },
		/// A name was cleared, and the given balance returned.
		NameCleared { pool: ID, deposit: BalanceOf<T> },
		/// A name was removed and the given balance slashed.
		NameKilled { target: ID, deposit: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// A name is too short.
		TooShort,
		/// A name is too long.
		TooLong,
		/// A pool isn't named.
		Unnamed,
	}

	/// The lookup table for names.
	#[pallet::storage]
	pub(super) type NameOf<T: Config> =
		StorageMap<_, Twox64Concat, ID, (BoundedVec<u8, T::MaxLength>, BalanceOf<T>)>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set a pool's name. The name should be a UTF-8-encoded string by convention, though
		/// we don't check it.
		///
		/// The name may not be more than `T::MaxLength` bytes, nor less than `T::MinLength` bytes.
		///
		/// If the pool doesn't already have a name, then a fee of `ReservationFee` is reserved
		/// in the account.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// # <weight>
		/// - O(1).
		/// - At most one balance operation.
		/// - One storage read/write.
		/// - One event.
		/// # </weight>
		// TODO: Create weight file for this.
		#[pallet::weight(0)]
		pub fn set_name(origin: OriginFor<T>, pool_id: ID, name: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let bounded_name: BoundedVec<_, _> =
				name.try_into().map_err(|()| Error::<T>::TooLong)?;
			ensure!(bounded_name.len() >= T::MinLength::get() as usize, Error::<T>::TooShort);

			let deposit = if let Some((_, deposit)) = <NameOf<T>>::get(&pool_id) {
				Self::deposit_event(Event::<T>::NameChanged { pool: pool_id });
				deposit
			} else {
				let deposit = T::ReservationFee::get();
				T::Currency::reserve(&sender, deposit)?;
				Self::deposit_event(Event::<T>::NameSet { pool: pool_id });
				deposit
			};

			<NameOf<T>>::insert(&pool_id, (bounded_name, deposit));
			Ok(())
		}

		/// Clear a pool's name and return the deposit. Fails if the pool was not named.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// # <weight>
		/// - O(1).
		/// - One balance operation.
		/// - One storage read/write.
		/// - One event.
		/// # </weight>
		#[pallet::weight(0)]
		pub fn clear_name(origin: OriginFor<T>, pool_id: ID) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let deposit = <NameOf<T>>::take(&pool_id).ok_or(Error::<T>::Unnamed)?.1;

			let err_amount = T::Currency::unreserve(&sender, deposit);
			debug_assert!(err_amount.is_zero());

			Self::deposit_event(Event::<T>::NameCleared { pool: pool_id, deposit });
			Ok(())
		}
	}
}


