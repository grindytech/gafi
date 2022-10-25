#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::{pallet_prelude::*, traits::Get, transactional, BoundedVec};
use frame_system::pallet_prelude::*;
use gafi_primitives::{
	constant::ID,
	membership::{Achievement, Achievements, Membership, MembershipLevelPoints},
	players::PlayerJoinedPoolStatistic,
};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub struct UpfrontPoolTimeAchievement<T, P> {
	pub phantom: PhantomData<(T, P)>,
	pub id: ID,
	pub min_joined_time: u128,
}

impl<T: frame_system::Config, Players: PlayerJoinedPoolStatistic<T::AccountId>>
	Achievement<T::AccountId> for UpfrontPoolTimeAchievement<T, Players>
{
	fn is_achieved(&self, sender: &T::AccountId) -> bool {
		Players::get_total_time_joined_upfront(sender) > self.min_joined_time
	}

	fn get_achievement_point(&self) -> u32 {
		10
	}
}
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Default, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxAchievement))]
pub struct MembershipInfo<MaxAchievement: Get<u32>> {
	pub is_reached: bool,
	pub level: u8,
	pub membership_point: u32,
	pub achievement_ids: BoundedVec<ID, MaxAchievement>,
}

#[frame_support::pallet]
pub mod pallet {

	use frame_support::traits::Currency;

	use super::*;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		/// Origin from which approvals must come.
		type ApproveOrigin: EnsureOrigin<Self::Origin>;

		type Players: PlayerJoinedPoolStatistic<Self::AccountId>;

		#[pallet::constant]
		type MaxMembers: Get<u32>;

		#[pallet::constant]
		type TotalMembershipLevel: Get<u32>;

		#[pallet::constant]
		type MaxAchievement: Get<u32>;

		type MembershipLevelPoints: MembershipLevelPoints<
			<Self as pallet::Config<I>>::TotalMembershipLevel,
		>;

		type Achievements: Achievements<
			UpfrontPoolTimeAchievement<Self, <Self as Config<I>>::Players>,
			<Self as pallet::Config<I>>::MaxAchievement,
		>;

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
	pub(super) type Members<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::AccountId, MembershipInfo<T::MaxAchievement>>;

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
			<Members<T, I>>::iter().for_each(|(member, membership_info)| {
				T::Achievements::get_membership_achievements().iter().for_each(|achievement| {
					let is_achieved = achievement.is_achieved(&member) &&
						!membership_info.achievement_ids.contains(&achievement.id);

					if is_achieved {
						let point = achievement.get_achievement_point();
						let new_point = membership_info.membership_point + point;
						let level = Self::get_level(new_point);
						let mut member_achievement_ids = membership_info.achievement_ids.clone();

						if member_achievement_ids.len() <
							T::MaxAchievement::get().try_into().unwrap()
						{
							member_achievement_ids.try_push(achievement.id).unwrap();
						}

						let new_membership_info = MembershipInfo::<T::MaxAchievement> {
							is_reached: false,
							level,
							membership_point: new_point,
							achievement_ids: member_achievement_ids,
						};

						<Members<T, I>>::insert(&member, new_membership_info);
						Self::deposit_event(Event::ReachMembershipLevel(1, member.clone()));
					}
				});
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

			let membership_info = MembershipInfo {
				is_reached: false,
				level: Default::default(),
				membership_point: Default::default(),
				achievement_ids: BoundedVec::default(),
			};

			<MemberCount<T, I>>::put(count + 1);
			<Members<T, I>>::insert(sender.clone(), membership_info);

			Self::deposit_event(Event::NewMember(count + 1, sender));

			Ok(())
		}

		#[pallet::weight(50_000_000)]
		#[transactional]
		pub fn remove_member(origin: OriginFor<T>, account_id: T::AccountId) -> DispatchResult {
			let _sender = T::ApproveOrigin::ensure_origin(origin)?;
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
		// fn is_added_point(sender: &T::AccountId) -> bool {}

		fn get_level(point: u32) -> u8 {
			let member_level_points = T::MembershipLevelPoints::get_membership_level_points();
			let result =
				member_level_points.into_iter().position(|level_point| point >= level_point);

			result.unwrap_or_default() as u8
		}
	}

	impl<T: Config<I>, I: 'static> Membership<T::AccountId> for Pallet<T, I> {
		fn is_registered(sender: &T::AccountId) -> bool {
			<Members<T, I>>::get(sender).is_some()
		}
	}
}
