#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use gafi_primitives::pool::{GafiPool, PlayerTicket, Service, Ticket, TicketType};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type UpfrontPool: GafiPool<Self::AccountId>;
		type StakingPool: GafiPool<Self::AccountId>;
		// type SponsoredPool: GafiPool<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Tickets<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Ticket<T::AccountId>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn join(origin: OriginFor<T>, ticket: TicketType) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			match ticket {
				TicketType::Upfront(level) => T::UpfrontPool::join(sender, level)?,
				TicketType::Staking(level) => T::StakingPool::join(sender, level)?,
				TicketType::Sponsored(_) => (),
			}
			Ok(())
		}
	}

	impl<T: Config> PlayerTicket<T::AccountId> for Pallet<T> {
		fn get_player_ticket(player: T::AccountId) -> Option<Ticket<T::AccountId>> {
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
}
