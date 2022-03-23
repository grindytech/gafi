#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

/*
* tx-handler pallet handle transaction fee
*
*/

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced, SignedImbalance,
			StoredMap, WithdrawReasons,
		},
		Twox64Concat,
	};
	use frame_system::pallet_prelude::*;
	use pallet_evm::AddressMapping;
	use pallet_evm::HashedAddressMapping;
	use pallet_evm::OnChargeEVMTransaction;
	use pallet_pool::pool::{AuroraZone, PackServiceProvider};
	use sp_core::crypto::AccountId32;
	use sp_core::{H160, U256};
	use sp_runtime::traits::{BlakeTwo256, DispatchInfoOf, Saturating, Zero};
	use utils::recover_signer;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	type NegativeImbalanceOf<C, T> =
		<C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config + pallet_pool::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type AuroraZone: AuroraZone<Self::AccountId>;
		type PackServiceProvider: PackServiceProvider<BalanceOf<Self>>;
		type OnChargeEVMTxHandler: OnChargeEVMTransaction<Self>;
		type AddressMapping: AddressMapping<Self::AccountId>;
	}

	// Errors.
	#[derive(PartialEq)]
	#[pallet::error]
	pub enum Error<T> {
		CanNotRecoverSigner,
		AddressNotCorrect,
		MessageNotCorrect,
		AccountAlreadyBind,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	// Storage
	#[pallet::storage]
	pub type Mapping<T: Config> = StorageMap<_, Twox64Concat, H160, AccountId32>;

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		[u8; 32]: From<<T as frame_system::Config>::AccountId>,
		AccountId32: From<<T as frame_system::Config>::AccountId>,
	{
		#[pallet::weight(100)]
		pub fn bind(
			origin: OriginFor<T>,
			signature: [u8; 65],
			address: H160,
		) -> DispatchResult {
			let sender: T::AccountId = ensure_signed(origin)?;
			Self::verify_owner(sender.clone(), signature, address)?;
			let account_id: AccountId32 = sender.into();
			<Mapping<T>>::insert(address, account_id);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		[u8; 32]: From<<T as frame_system::Config>::AccountId>,
	{
		/*
			make sure the signature belong to the address
			make sure the message is the keccak_256 of sender
		*/
		pub fn verify_owner(
			sender: T::AccountId,
			signature: [u8; 65],
			address: H160,
		) -> Result<(), Error<T>> {
			ensure!(Mapping::<T>::get(address) == None, <Error<T>>::AccountAlreadyBind);
			let account_bytes: [u8; 32] = sender.into();
			let hash_message = sp_io::hashing::keccak_256(&account_bytes);
			let signer = recover_signer(signature, hash_message);
			ensure!(signer != None, <Error<T>>::CanNotRecoverSigner);
			ensure!(address == signer.unwrap(), <Error<T>>::AddressNotCorrect);
			Ok(())
		}
	}

	pub struct AurCurrencyAdapter<C, OU>(sp_std::marker::PhantomData<(C, OU)>);

	impl<T, C, OU> OnChargeEVMTransaction<T> for AurCurrencyAdapter<C, OU>
	where
		T: Config,
		C: Currency<<T as frame_system::Config>::AccountId>,
		C::PositiveImbalance: Imbalance<
			<C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
			Opposite = C::NegativeImbalance,
		>,
		C::NegativeImbalance: Imbalance<
			<C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
			Opposite = C::PositiveImbalance,
		>,
		OU: OnUnbalanced<NegativeImbalanceOf<C, T>>,
	{
		type LiquidityInfo = <<T as pallet::Config>::OnChargeEVMTxHandler as OnChargeEVMTransaction<T>>::LiquidityInfo;
		fn withdraw_fee(
			who: &H160,
			fee: U256,
		) -> Result<Self::LiquidityInfo, pallet_evm::Error<T>> {
			let account_id = <T as pallet::Config>::AddressMapping::into_account_id(*who);
			let mut service_fee = fee;
			if let Some(player) = T::AuroraZone::is_in_aurora_zone(&account_id) {
				if let Some(service) = T::PackServiceProvider::get_service(player.service) {
					service_fee = fee / service.discount;
				}
			}
			T::OnChargeEVMTxHandler::withdraw_fee(who, service_fee)
		}

		fn correct_and_deposit_fee(
			who: &H160,
			corrected_fee: U256,
			already_withdrawn: Self::LiquidityInfo,
		) {
			T::OnChargeEVMTxHandler::correct_and_deposit_fee(who, corrected_fee, already_withdrawn)
		}

		fn pay_priority_fee(tip: U256) {
			T::OnChargeEVMTxHandler::pay_priority_fee(tip)
		}
	}

	pub struct ProofAddressMapping<T>(sp_std::marker::PhantomData<T>);

	impl<T> pallet_evm::AddressMapping<AccountId32> for ProofAddressMapping<T>
	where
		T: Config,
	{
		fn into_account_id(address: H160) -> AccountId32 {
			if let Some(account_id) = Mapping::<T>::get(address) {
				account_id
			} else {
				HashedAddressMapping::<BlakeTwo256>::into_account_id(address)
			}
		}
	}
}
