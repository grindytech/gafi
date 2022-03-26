#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::{
		fungible::Inspect, Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced,
		OriginTrait, SignedImbalance, StoredMap, WithdrawReasons,
	},
	Twox64Concat,
};
use frame_system::{pallet_prelude::*, Origin, RawOrigin};
pub use pallet::*;
use pallet_evm::AddressMapping;
use pallet_evm::HashedAddressMapping;
use pallet_evm::OnChargeEVMTransaction;
use sp_core::{H160, U256};
use sp_runtime::traits::{BlakeTwo256, DispatchInfoOf, Saturating, StaticLookup, Zero};
use utils::{eth_recover, to_ascii_hex, EcdsaSignature, EthereumAddress};

use pallet_pool::pool::{AuroraZone, PackServiceProvider};
use sp_core::crypto::AccountId32;
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
		type AuroraZone: AuroraZone<Self::AccountId>;
		type PackServiceProvider: PackServiceProvider<BalanceOf<Self>>;
		type OnChargeEVMTxHandler: OnChargeEVMTransaction<Self>;
		type AddressMapping: AddressMapping<Self::AccountId>;
		type DefaultAddressMapping: AddressMapping<Self::AccountId>;
		#[pallet::constant]
		type MessagePrefix: Get<&'static [u8]>;
	}

	// Errors.
	#[derive(PartialEq)]
	#[pallet::error]
	pub enum Error<T> {
		SignatureOrAddressNotCorrect,
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
			withdraw: bool,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Mapping::<T>::get(address) == None, <Error<T>>::AccountAlreadyBind);
			ensure!(
				Self::verify_bind(sender.clone(), signature, address.to_fixed_bytes()),
				<Error<T>>::SignatureOrAddressNotCorrect,
			);

			if withdraw {
				Self::transfer_all(address, sender.clone(), true)?;
			}
			let account_id: AccountId32 = sender.into();
			<Mapping<T>>::insert(address, account_id);
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T>
where
	[u8; 32]: From<<T as frame_system::Config>::AccountId>,
{
	pub fn verify_bind(sender: T::AccountId, sig: [u8; 65], address: [u8; 20]) -> bool {
		let sig_converter = EcdsaSignature(sig);
		let address_convert = EthereumAddress(address);
		let who = sender.using_encoded(to_ascii_hex);
		let signer = eth_recover(&sig_converter, &who, &[][..], T::MessagePrefix::get());
		signer == Some(address_convert)
	}

	pub fn transfer_all(from: H160, to: T::AccountId, keep_alive: bool) -> DispatchResult {
		let from_account: T::AccountId = T::DefaultAddressMapping::into_account_id(from);
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
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	fn into_account_id(address: H160) -> AccountId32 {
		if let Some(account_id) = Mapping::<T>::get(address) {
			account_id
		} else {
			T::DefaultAddressMapping::into_account_id(address).into()
		}
	}
}
