#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

use frame_support::pallet_prelude::*;
use frame_support::traits::{
	fungible::Inspect, Currency, ExistenceRequirement, ReservableCurrency,
};
use frame_system::pallet_prelude::*;
use pallet_evm::AddressMapping;
use pallet_evm::GetContractCreator;
use sp_core::H160;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

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

		type ContractCreator: GetContractCreator;
	}

	//** STORAGE  **//
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type ContractOwner<T: Config> = StorageMap<_, Twox64Concat, H160, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		NotContractOwner,
		ContractNotFound,
		ContractClaimed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn claim_contract(origin: OriginFor<T>, contract: H160) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(
				ContractOwner::<T>::get(contract).is_none(),
				<Error<T>>::ContractClaimed
			);
			Self::verify_owner(&sender, &contract)?;
			ContractOwner::<T>::insert(contract, sender.clone());
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn verify_owner(sender: &T::AccountId, contract: &H160) -> Result<(), Error<T>> {
			let contract_creator = Self::get_contract_creator(&contract)?;
			if *sender != contract_creator {
				return Err(Error::<T>::NotContractOwner);
			}
			Ok(())
		}

		fn get_contract_creator(contract: &H160) -> Result<T::AccountId, Error<T>> {
			match T::ContractCreator::get_contract_creator(contract) {
				Some(address) => return Ok(T::AddressMapping::into_account_id(address)),
				None => Err(Error::<T>::ContractNotFound),
			}
		}
	}
}
