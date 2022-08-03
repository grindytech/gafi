use frame_support::{traits::Get, BoundedVec};

pub trait Membership<AccountId> {
	fn is_registered(sender: &AccountId) -> bool;
}

impl<AccountId> Membership<AccountId> for () {
	fn is_registered(_sender: &AccountId) -> bool {
		Default::default()
	}
}

pub trait Achievements<T, MaxAchievement: Get<u32>> {
	fn get_membership_achievements() -> BoundedVec<T, MaxAchievement>;
}

impl<T, MaxAchievement: Get<u32>> Achievements<T, MaxAchievement> for () {
	fn get_membership_achievements() -> BoundedVec<T, MaxAchievement> {
		Default::default()
	}
}

pub trait Achievement<AccountId> {
	fn is_achieved(&self, sender: &AccountId) -> bool;

	fn get_achievement_point(&self) -> u32;
}

pub trait MembershipLevelPoints<TotalMembershipLevel: Get<u32>> {
	fn get_membership_level_points() -> BoundedVec<u32, TotalMembershipLevel>;
}

impl<TotalMembershipLevel: Get<u32>> MembershipLevelPoints<TotalMembershipLevel> for () {
	fn get_membership_level_points() -> BoundedVec<u32, TotalMembershipLevel> {
		Default::default()
	}
}
