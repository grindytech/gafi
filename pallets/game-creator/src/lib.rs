#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::traits::{
	BalanceStatus, Currency, ReservableCurrency,
};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use pallet_evm::{AddressMapping, GetContractCreator};
use sp_core::H160;
use gafi_primitives::game_creator::GetGameCreator;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type AddressMapping: AddressMapping<Self::AccountId>;

		#[pallet::constant]
		type MaxContractOwned: Get<u32>;

		type Currency: ReservableCurrency<Self::AccountId>;

		type ContractCreator: GetContractCreator;

		#[pallet::constant]
		type ReservationFee: Get<BalanceOf<Self>>;
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
			<T as pallet::Config>::Currency::reserve(&sender, T::ReservationFee::get())?;
			ContractOwner::<T>::insert(contract, sender.clone());
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn change_ownership(
			origin: OriginFor<T>,
			contract: H160,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::verify_owner(&sender, &contract)?;

			<T as pallet::Config>::Currency::repatriate_reserved(
				&sender,
				&new_owner,
				T::ReservationFee::get(),
				BalanceStatus::Reserved,
			)?;

			ContractOwner::<T>::insert(contract, new_owner);
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn withdraw_contract(origin: OriginFor<T>, contract: H160) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::verify_owner(&sender, &contract)?;

			ContractOwner::<T>::remove(contract);
			<T as pallet::Config>::Currency::unreserve(&sender, T::ReservationFee::get());
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn verify_owner(sender: &T::AccountId, contract: &H160) -> Result<(), Error<T>> {
			if let Some(owner) = ContractOwner::<T>::get(&contract) {
				if owner != *sender {
					return Err(Error::<T>::NotContractOwner);
				}
			} else {
				let contract_creator = Self::get_contract_creator(&contract)?;
				if contract_creator != *sender {
					return Err(Error::<T>::NotContractOwner);
				}
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

	impl<T: Config> GetGameCreator<T::AccountId> for Pallet<T> {
		fn get_game_creator(contract: &H160) -> Option<T::AccountId> {
			ContractOwner::<T>::get(contract)
		}
	}
}
