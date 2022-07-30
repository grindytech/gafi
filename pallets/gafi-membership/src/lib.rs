#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

use frame_support::{pallet_prelude::*, traits::Get, transactional};
use frame_system::pallet_prelude::*;
use gafi_primitives::{membership::Membership, players::PlayerJoinedPoolStatistic};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct MembershipInfo {
	pub is_reached: bool,
}

#[frame_support::pallet]
pub mod pallet {

	use super::*;

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
		StorageMap<_, Twox64Concat, T::AccountId, MembershipInfo>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		NewMember(u32, T::AccountId),
		RemoveMember(u32, T::AccountId),
		ReachMembershipLevel(u32, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		AlreadyRegistered,
		ExceedMaxMembers,
		MemberNotExist,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn on_finalize(_block_number: BlockNumberFor<T>) {
			<Members<T, I>>::iter().for_each(|(member, _)| {
				let is_reached = Self::is_reach_membership_discount(&member);
				if is_reached {
					let membership_info = MembershipInfo { is_reached };

					<Members<T, I>>::insert(&member, membership_info);
					Self::deposit_event(Event::ReachMembershipLevel(1, member));
				}
			});
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::weight(50_000_000)]
		#[transactional]
		pub fn registration(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let count = Self::member_count();

			ensure!(
				!Self::is_registered(&sender),
				Error::<T, I>::AlreadyRegistered
			);

			ensure!(
				count < T::MaxMembers::get(),
				Error::<T, I>::ExceedMaxMembers
			);

			let membership_info = MembershipInfo { is_reached: false };

			<MemberCount<T, I>>::put(count + 1);
			<Members<T, I>>::insert(sender.clone(), membership_info);

			Self::deposit_event(Event::NewMember(count + 1, sender));

			Ok(())
		}

		#[pallet::weight(50_000_000)]
		#[transactional]
		pub fn remove_member(origin: OriginFor<T>, account_id: T::AccountId) -> DispatchResult {
			let sender = T::ApproveOrigin::ensure_origin(origin)?;
			let count = Self::member_count();

			ensure!(
				<Members<T, I>>::get(account_id.clone()).is_some(),
				Error::<T, I>::MemberNotExist
			);

			<MemberCount<T, I>>::put(count - 1);
			<Members<T, I>>::remove(account_id.clone());

			Self::deposit_event(Event::RemoveMember(count - 1, account_id));

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		fn is_reach_membership_discount(sender: &T::AccountId) -> bool {
			T::Players::get_total_time_joined_upfront(sender) > T::MinJoinTime::get()
		}
	}

	impl<T: Config<I>, I: 'static> Membership<T::AccountId> for Pallet<T, I> {
		fn is_registered(sender: &T::AccountId) -> bool {
			<Members<T, I>>::get(sender).is_some()
		}
	}
}
