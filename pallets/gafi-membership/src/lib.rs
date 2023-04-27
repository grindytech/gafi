// This file is part of Gafi Network.

// Copyright (C) 2021-2023 Grindy Technologies.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use crate::weights::WeightInfo;
use frame_support::{pallet_prelude::*, traits::Get, transactional, BoundedVec};
use frame_system::pallet_prelude::*;
use gafi_support::{
	common::ID,
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

pub mod weights;
pub use weights::*;

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
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism.
		type Currency: Currency<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Origin from which approvals must come.
		type ApproveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		type Players: PlayerJoinedPoolStatistic<Self::AccountId>;

		/// Max members that allow to join membership program
		#[pallet::constant]
		type MaxMembers: Get<u32>;

		/// Max level that user can reach
		#[pallet::constant]
		type TotalMembershipLevel: Get<u32>;

		/// Max achievement that membership program will have
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

	//** STORAGE  **//
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	/// Number of current members in membership program
	#[pallet::storage]
	#[pallet::getter(fn member_count)]
	pub type MemberCount<T, I = ()> = StorageValue<_, u32, ValueQuery>;

	/// Holding the members info
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
		/// User already in membership program
		AlreadyRegistered,

		/// Reach max members of the program
		ExceedMaxMembers,

		/// Member not exist in the list member of program
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
		/// User register to the membership program
		///
		/// The origin must be Signed
		///
		/// Emits `NewMember` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::registration(50u64))]
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

		/// Remove member out of the membership program
		///
		/// The origin must be Signed
		///
		/// /// Parameters:
		/// - `account_id`: Account Id
		///
		/// Emits `NewMember` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config<I>>::WeightInfo::remove_member(50u64))]
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
