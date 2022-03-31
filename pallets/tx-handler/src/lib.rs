#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{fungible::Inspect, Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced},
	Twox64Concat,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use pallet_evm::AddressMapping;
use pallet_evm::OnChargeEVMTransaction;
use sp_core::{H160, U256};
use utils::{eth_recover, to_ascii_hex, EcdsaSignature, EthereumAddress};

use pallet_pool::pool::{AuroraZone, PackServiceProvider};
use sp_core::crypto::AccountId32;
use sp_std::{if_std};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::weights::WeightInfo;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	pub type NegativeImbalanceOf<C, T> =
		<C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config + pallet_balances::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type WeightInfo: WeightInfo;
		type AuroraZone: AuroraZone<Self::AccountId>;
		type PackServiceProvider: PackServiceProvider<BalanceOf<Self>>;
		type OnChargeEVMTxHandler: OnChargeEVMTransaction<Self>;
		type AddressMapping: AddressMapping<Self::AccountId>;
		#[pallet::constant]
		type MessagePrefix: Get<&'static [u8]>;
	}

	// Errors.
	#[derive(PartialEq)]
	#[pallet::error]
	pub enum Error<T> {
		SignatureOrAddressNotCorrect,
		EVMAccountAlreadyBond,
		AuroraAccountAlreadyBond,

		NonbondAccount,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	// Storage
	#[pallet::storage]
	pub type H160Mapping<T: Config> = StorageMap<_, Twox64Concat, H160, AccountId32>;

	#[pallet::storage]
	pub type Id32Mapping<T: Config> = StorageMap<_, Twox64Concat, AccountId32, H160>;

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		[u8; 32]: From<<T as frame_system::Config>::AccountId>,
		AccountId32: From<<T as frame_system::Config>::AccountId>,
	{
		#[pallet::weight(<T as pallet::Config>::WeightInfo::bond(100u32))]
		pub fn bond(
			origin: OriginFor<T>,
			signature: [u8; 65],
			address: H160,
			withdraw: bool,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let account_id: AccountId32 = sender.clone().into();

			ensure!(H160Mapping::<T>::get(address) == None, <Error<T>>::EVMAccountAlreadyBond);
			ensure!(
				Id32Mapping::<T>::get(account_id.clone()) == None,
				<Error<T>>::AuroraAccountAlreadyBond
			);
			ensure!(
				Self::verify_bond(sender.clone(), signature, address.to_fixed_bytes()),
				<Error<T>>::SignatureOrAddressNotCorrect,
			);
			if withdraw {
				Self::transfer_all(address, sender.clone(), true)?;
			}

			Self::insert_origin_pair_bond(address, account_id.clone());
			Self::insert_pair_bond(address, account_id);
			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::unbond(100u32))]
		pub fn unbond(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let account_id: AccountId32 = sender.into();

			let evm_address = <Id32Mapping<T>>::get(account_id);
			ensure!(evm_address != None, <Error<T>>::NonbondAccount);
			let id32_address = <H160Mapping<T>>::get(evm_address.unwrap());
			ensure!(id32_address != None, <Error<T>>::NonbondAccount);

			Self::remove_pair_bond(evm_address.unwrap(), id32_address.unwrap());
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T>
where
	[u8; 32]: From<<T as frame_system::Config>::AccountId>,
{
	pub fn verify_bond(sender: T::AccountId, sig: [u8; 65], address: [u8; 20]) -> bool {
		let sig_converter = EcdsaSignature(sig);
		let address_convert = EthereumAddress(address);
		let who = sender.using_encoded(to_ascii_hex);
		let signer = eth_recover(&sig_converter, &who, &[][..], T::MessagePrefix::get());
		signer == Some(address_convert)
	}

	pub fn transfer_all(from: H160, to: T::AccountId, keep_alive: bool) -> DispatchResult {
		let from_account: T::AccountId =
			<T as pallet::Config>::AddressMapping::into_account_id(from);
		let reducible_balance: u128 =
			pallet_balances::pallet::Pallet::<T>::reducible_balance(&from_account, keep_alive)
				.try_into()
				.ok()
				.unwrap();
		let existence = if keep_alive {
			ExistenceRequirement::KeepAlive
		} else {
			ExistenceRequirement::AllowDeath
		};
		<T as pallet::Config>::Currency::transfer(
			&from_account,
			&to,
			reducible_balance.try_into().ok().unwrap(),
			existence,
		)
	}

	fn into_h160(account_id: AccountId32) -> H160 {
		let mut origin_address = H160::default();
		let data: [u8; 32] = account_id.into();
		if data.starts_with(b"evm:") {
			origin_address = H160::from_slice(&data[4..24]);
		} else {
			origin_address = H160::from_slice(&data[0..20]);
		}
		origin_address
	}

	fn insert_origin_pair_bond(address: H160, account_id: AccountId32)
	where
		sp_runtime::AccountId32: From<<T as frame_system::Config>::AccountId>,
	{
		let origin_account: T::AccountId =
			<T as pallet::Config>::AddressMapping::into_account_id(address);
		let origin_account_id: AccountId32 = origin_account.into();
		let origin_address: H160 = Self::into_h160(account_id);

		<H160Mapping<T>>::insert(origin_address, origin_account_id.clone());
		<Id32Mapping<T>>::insert(origin_account_id, origin_address);
	}

	fn insert_pair_bond(address: H160, account_id: AccountId32) {
		<H160Mapping<T>>::insert(address, account_id.clone());
		<Id32Mapping<T>>::insert(account_id, address);
	}

	fn remove_pair_bond(address: H160, account_id: AccountId32) {
		<H160Mapping<T>>::remove(address);
		<Id32Mapping<T>>::remove(account_id);
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
	type LiquidityInfo =
		<<T as pallet::Config>::OnChargeEVMTxHandler as OnChargeEVMTransaction<T>>::LiquidityInfo;
	fn withdraw_fee(who: &H160, fee: U256) -> Result<Self::LiquidityInfo, pallet_evm::Error<T>> {
		T::OnChargeEVMTxHandler::withdraw_fee(who, fee)
	}

	fn correct_and_deposit_fee(
		who: &H160,
		corrected_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	) {
		let mut service_fee = corrected_fee;
		let account_id = <T as pallet::Config>::AddressMapping::into_account_id(*who);
		if let Some(player) = T::AuroraZone::is_in_aurora_zone(&account_id) {
			if let Some(service) = T::PackServiceProvider::get_service(player.service) {
				service_fee = corrected_fee - (corrected_fee * service.discount / 100);
			}
		}

		if_std! {
			println!("service_fee: {:?}", service_fee);
		}

		T::OnChargeEVMTxHandler::correct_and_deposit_fee(who, service_fee, already_withdrawn)
	}

	fn pay_priority_fee(tip: U256) {
		T::OnChargeEVMTxHandler::pay_priority_fee(tip)
	}
}
pub struct ProofAddressMapping<T>(sp_std::marker::PhantomData<T>);
pub struct DefaultAddressMapping;

impl pallet_evm::AddressMapping<AccountId32> for DefaultAddressMapping {
	fn into_account_id(address: H160) -> AccountId32 {
		let mut data: [u8; 32] = [0u8; 32];
		data[0..4].copy_from_slice(b"evm:");
		data[4..24].copy_from_slice(&address[..]);
		AccountId32::from(data)
	}
}

impl<T> pallet_evm::AddressMapping<AccountId32> for ProofAddressMapping<T>
where
	T: Config,
{
	fn into_account_id(address: H160) -> AccountId32 {
		if let Some(account_id) = H160Mapping::<T>::get(address) {
			account_id
		} else {
			DefaultAddressMapping::into_account_id(address)
		}
	}
}
