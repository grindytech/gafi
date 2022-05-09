#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

use frame_support::pallet_prelude::*;
use frame_support::traits::{
	fungible::Inspect, Currency, ExistenceRequirement, ReservableCurrency,
};
use frame_system::pallet_prelude::*;
use pallet_evm::AddressMapping;
use sp_core::H160;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type AddressMapping: AddressMapping<Self::AccountId>;

		#[pallet::constant]
		type MaxContractOwned: Get<u32>;

		type Currency: ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type ContractOwned<T: Config> = StorageMap<_, Twox64Concat, H160, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn contract_mapping)]
	pub(super) type ContractMapping<T: Config> = StorageMap<_, Twox64Concat, H160, H160>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MappedContract {contract: H160, owner: H160},
	}

	#[pallet::error]
	pub enum Error<T> {
		NotContractOwner,
		ExceedMaxContractOwned,
		ContractNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn claim_reward(origin: OriginFor<T>, contract: H160) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::is_owner(&sender, &contract)?;

			let contract_acc = T::AddressMapping::into_account_id(contract);

			Self::transfer_all(&contract_acc, &sender, false)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn is_owner(sender: &T::AccountId, contract: &H160) -> Result<(), Error<T>> {
			if let Some(owner) = ContractOwned::<T>::get(contract) {
				if owner == *sender {
					return Ok(());
				}
				return Err(<Error<T>>::NotContractOwner);
			}
			return Err(<Error<T>>::ContractNotFound);
		}

		pub fn mapping_contract(contract: &H160, owner: &H160) {
			// <ContractMapping<T>>::insert(contract.clone(), owner.clone());
			Self::deposit_event(Event::MappedContract{contract: contract.clone(), owner: owner.clone()});
		}

		pub fn transfer_all(
			from: &T::AccountId,
			to: &T::AccountId,
			keep_alive: bool,
		) -> DispatchResult {
			let reducible_balance: u128 =
				pallet_balances::pallet::Pallet::<T>::reducible_balance(from, keep_alive)
					.try_into()
					.ok()
					.unwrap();
			let existence = if keep_alive {
				ExistenceRequirement::KeepAlive
			} else {
				ExistenceRequirement::AllowDeath
			};
			<T as pallet::Config>::Currency::transfer(
				from,
				to,
				reducible_balance.try_into().ok().unwrap(),
				existence,
			)
		}
	}
}
