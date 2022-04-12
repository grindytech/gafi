#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use gafi_primitives::pool::{GafiPool, PlayerTicket, Service, MasterPool, TicketType};
use frame_support::traits::Currency;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type UpfrontPool: GafiPool<Self::AccountId>;
		type StakingPool: GafiPool<Self::AccountId>;
		// type SponsoredPool: GafiPool<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Tickets<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, TicketType>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Joined { sender: T::AccountId, ticket: TicketType },
		Leaved { sender: T::AccountId, ticket: TicketType },
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyJoined,
		NotFoundInPool,
		ComingSoon,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn join(origin: OriginFor<T>, ticket: TicketType) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Tickets::<T>::get(sender.clone()) == None, <Error<T>>::AlreadyJoined);

			match ticket {
				TicketType::Upfront(level) => T::UpfrontPool::join(sender.clone(), level)?,
				TicketType::Staking(level) => T::StakingPool::join(sender.clone(), level)?,
				TicketType::Sponsored(_) => {
					return Err(Error::<T>::ComingSoon.into());
				},
			}

			Tickets::<T>::insert(sender.clone(), ticket);
			Self::deposit_event(Event::<T>::Joined { sender, ticket });
			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn leave(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			if let Some(ticket) = Tickets::<T>::get(sender.clone()) {
				match ticket {
					TicketType::Upfront(_) => T::UpfrontPool::leave(sender.clone())?,
					TicketType::Staking(_) => T::StakingPool::leave(sender.clone())?,
					TicketType::Sponsored(_) => {
						return Err(Error::<T>::ComingSoon.into());
					},
				}
				Tickets::<T>::remove(sender.clone());
				Self::deposit_event(Event::<T>::Leaved { sender: sender, ticket: ticket});
				Ok(())
			} else {
				return Err(Error::<T>::NotFoundInPool.into());
			}
		}
	}

	impl<T: Config> PlayerTicket<T::AccountId> for Pallet<T> {
		fn get_player_ticket(player: T::AccountId) -> Option<TicketType> {
			Tickets::<T>::get(player)
		}

		fn get_ticket(ticket: TicketType) -> Service {
			match ticket {
				TicketType::Upfront(level) => T::UpfrontPool::get_service(level),
				TicketType::Staking(level) => T::StakingPool::get_service(level),
				TicketType::Sponsored(_) => todo!(),
			}
		}
	}

	impl<T: Config> MasterPool<T::AccountId> for Pallet<T> {
		fn remove_player(player: &T::AccountId) {
			Tickets::<T>::remove(&player);
		}
	}
}
