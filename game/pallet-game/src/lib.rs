#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use sp_core::U256;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::traits::{
	tokens::nonfungibles_v2::{Create, Mutate, Transfer},
	Currency, Randomness, ReservableCurrency,
};
use frame_system::Config as SystemConfig;
use gafi_support::{common::constant::ID, game::GameSetting};
use sp_core::blake2_256;
use sp_runtime::traits::StaticLookup;

pub type DepositBalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

type AccountIdLookupOf<T> = <<T as SystemConfig>::Lookup as StaticLookup>::Source;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use gafi_support::{game::GameProvider, common::types::BlockNumber};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(_);

	pub type BalanceOf<T, I = ()> =
		<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism, used for paying for reserves.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// generate random ID
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// The type used to identify a unique game
		type GameId: Member + Parameter + MaxEncodedLen + Copy;

		/// Max name length
		type MaxNameLength: Get<u32>;

		/// Min name length
		type MinNameLength: Get<u32>;

		/// Max Swapping Fee
		type MaxSwapFee: Get<u8>;
	}

	/// Store basic game info
	#[pallet::storage]
	pub(super) type Games<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::GameId, (T::AccountId, BoundedVec<u8, T::MaxNameLength>)>;

	#[pallet::storage]
	pub(super) type SwapFee<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::GameId, (u8, BlockNumber)>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		NotGameOwner,
		GameIdNotFound,
		NameTooLong,
		NameTooShort,
		SwapFeeTooHigh,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {}

	impl<T: Config<T>, I: 'static> Pallet<T, I> {
		pub fn gen_id() -> Result<ID, Error<T>> {
			let payload = (
				T::Randomness::random(&b""[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			Ok(payload.using_encoded(blake2_256))
		}
	}

	impl<T: Config<I>, I: 'static> GameSetting<Error<T>, T::AccountId, T::GameId> for Pallet<T, I> {
		fn create_game(id: T::GameId, owner: T::AccountId, name: Vec<u8>) -> Result<T::GameId, Error<T>> {

			let bounded_name: BoundedVec<_, _> =
				name.try_into().map_err(|_| Error::<T>::NameTooLong)?;
			ensure!(bounded_name.len() >= T::MinNameLength::get() as usize, Error::<T>::NameTooShort);

			Games::<T, I>::insert(id, (owner, bounded_name));
			Ok(id)
		}

		fn set_swapping_fee(id: T::GameId, fee: u8, start_block: BlockNumber) -> Result<(), Error<T>> {
			ensure!(fee <= T::MaxSwapFee::get(), Error::SwapFeeTooHigh);
			SwapFee::<T, I>::insert(id, (fee, start_block));
			Ok(())
		}

		fn freeze_collection_transfer() {
			todo!()
		}
		fn freeze_collection_swap() {
			todo!()
		}
		fn freeze_item_transfer() {
			todo!()
		}
		fn freeze_item_swap() {
			todo!()
		}
	}

	impl<T: Config<I>, I: 'static> GameProvider<Error<T>, T::AccountId, T::GameId> for Pallet<T, I> {
		fn get_swap_fee() -> Result<u8, Error<T>> {
			todo!()
		}

		fn is_game_owner(id: T::GameId, owner: T::AccountId) -> Result<(), Error<T>> {
			if let Some(game) = Games::<T, I>::get(id) {
				if game.0 == owner {
					Ok(())
				} else {
					Err(Error::<T>::NotGameOwner)
				}
			} else {
				Err(Error::<T>::GameIdNotFound)
			}
		}
	}
}
