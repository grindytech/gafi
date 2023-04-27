#![cfg_attr(not(feature = "std"), no_std)]

use crate::player::Player;
use codec::Encode;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, Randomness},
};
use frame_system::pallet_prelude::*;
use gafi_support::{
	common::ID, pallet::players::PlayerJoinedPoolStatistic,
	pool::SystemPool,
};
use pallet_timestamp::{self as timestamp};
use sp_io::hashing::blake2_256;
use sp_runtime::traits::StaticLookup;

pub use pallet::*;
#[cfg(test)]
mod mock;
mod player;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	pub type NAME = [u8; 16];

	pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config + timestamp::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		type GameRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		type UpfrontPool: SystemPool<AccountIdLookupOf<Self>, Self::AccountId>;

		type StakingPool: SystemPool<AccountIdLookupOf<Self>, Self::AccountId>;
	}

	// Errors.
	#[derive(PartialEq)]
	#[pallet::error]
	pub enum Error<T> {
		PlayerIdUsed,
		PlayerExisted,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewPlayerCreated(T::AccountId, ID),
	}

	// Storage
	#[pallet::storage]
	pub(super) type Players<T: Config> = StorageMap<_, Twox64Concat, ID, Player<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn player_owned)]
	pub type PlayerOwned<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, ID>;

	#[pallet::storage]
	#[pallet::getter(fn total_time_joined_upfront)]
	pub type TotalTimeJoinedUpfront<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u128>;

	#[pallet::storage]
	#[pallet::getter(fn total_time_joined_staking)]
	pub type TotalTimeJoinedStaking<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u128>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(100)]
		pub fn create_player(origin: OriginFor<T>, name: NAME) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let id = Self::create_new_player(sender.clone(), name)?;
			Self::deposit_event(Event::NewPlayerCreated(sender, id));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn gen_id() -> Result<ID, Error<T>> {
			let payload = (
				T::GameRandomness::random(&b""[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			Ok(payload.using_encoded(blake2_256))
		}

		pub fn create_new_player(sender: T::AccountId, user_name: NAME) -> Result<ID, Error<T>> {
			ensure!(
				Self::player_owned(sender.clone()).is_none(),
				<Error<T>>::PlayerExisted
			);
			let id = Self::gen_id()?;
			ensure!(Players::<T>::get(id).is_none(), <Error<T>>::PlayerIdUsed);
			let player = Player::<T::AccountId> {
				id,
				owner: sender.clone(),
				name: user_name,
			};

			<Players<T>>::insert(id, player);
			<PlayerOwned<T>>::insert(sender, id);
			Ok(id)
		}

		fn moment_to_u128(input: T::Moment) -> u128 {
			sp_runtime::SaturatedConversion::saturated_into(input)
		}
	}

	impl<T: Config> PlayerJoinedPoolStatistic<T::AccountId> for Pallet<T> {
		fn get_total_time_joined_upfront(player: &T::AccountId) -> u128 {
			let current_joined_time = TotalTimeJoinedUpfront::<T>::get(player).unwrap_or(0u128);

			if let Some(ticket) = T::UpfrontPool::get_ticket(player) {
				let join_time = ticket.join_time;
				let now = Self::moment_to_u128(<timestamp::Pallet<T>>::get());

				return now.saturating_sub(join_time).saturating_add(current_joined_time)
			}

			current_joined_time
		}

		fn get_total_time_joined_staking(player: &T::AccountId) -> u128 {
			let current_joined_time = TotalTimeJoinedStaking::<T>::get(player).unwrap_or(0u128);

			if let Some(ticket) = T::StakingPool::get_ticket(player) {
				let join_time = ticket.join_time;
				let now = Self::moment_to_u128(<timestamp::Pallet<T>>::get());

				return now.saturating_sub(join_time).saturating_add(current_joined_time)
			}

			current_joined_time
		}
	}
}
