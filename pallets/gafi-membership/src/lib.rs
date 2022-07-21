#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

use gafi_primitives::players::PlayerJoinedPoolStatistic;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{pallet_prelude::*, traits::Get, transactional, BoundedVec};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

		/// Origin from which approvals must come.
		type ApproveOrigin: EnsureOrigin<Self::Origin>;

		type Players: PlayerJoinedPoolStatistic<Self::AccountId>;

		#[pallet::constant]
		type MaxMembers: Get<u32>;

		#[pallet::constant]
		type MinJoinTime: Get<u128>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::storage]
	#[pallet::getter(fn member_count)]
	pub type MemberCount<T, I = ()> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxMembers>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		NewMember(u32, T::AccountId),
		RemoveMember(u32, T::AccountId)
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		ExceedMaxMembers,
		MemberNotExist,
		NotEligibleForRegistration
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {

		#[pallet::weight(50_000_000)]
		#[transactional]
		pub fn registration(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let count = Self::member_count();

			// Check if user eligible for join membership
			ensure!(T::Players::get_total_time_joined_upfront(sender.clone()) > T::MinJoinTime::get(), Error::<T, I>::NotEligibleForRegistration);

			let mut members = <Members<T, I>>::get();
			members
				.try_push(sender.clone())
				.map_err(|_| Error::<T, I>::ExceedMaxMembers)?;


			<MemberCount<T, I>>::put(count + 1);
			<Members<T, I>>::put(members);

			Self::deposit_event(Event::NewMember(count + 1, sender));

			Ok(())
		}

		#[pallet::weight(50_000_000)]
		#[transactional]
		pub fn remove_member(origin: OriginFor<T>, account_id: T::AccountId) -> DispatchResult {
			let sender = T::ApproveOrigin::ensure_origin(origin)?;
			let count = Self::member_count();

			let new_members: Vec<T::AccountId> = <Members<T, I>>::get()
			.into_iter()
			.filter(|member| *member != account_id).collect();

			ensure!(<MemberCount<T, I>>::get() as usize != new_members.len(), Error::<T, I>::MemberNotExist);

			let bounded_new_member = BoundedVec::try_from(new_members).unwrap();

			<MemberCount<T, I>>::put(count - 1);
			<Members<T, I>>::put(bounded_new_member);

			Self::deposit_event(Event::RemoveMember(count - 1, account_id));

			Ok(())
		}
	}
}
