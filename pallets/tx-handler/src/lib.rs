#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, Imbalance, OnUnbalanced},
};
pub use pallet::*;
use pallet_evm::AddressMapping;
use pallet_evm::OnChargeEVMTransaction;
use sp_core::{H160, U256};
use pallet_staking_pool::StakingPool;
use pallet_option_pool::pool::{AuroraZone, PackServiceProvider};

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
		type StakingPool: StakingPool<Self::AccountId>;
		type PackServiceProvider: PackServiceProvider<BalanceOf<Self>>;
		type OnChargeEVMTxHandler: OnChargeEVMTransaction<Self>;
		type AddressMapping: AddressMapping<Self::AccountId>;
	}

	// Errors.
	#[derive(PartialEq)]
	#[pallet::error]
	pub enum Error<T> {
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	// Storage

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
				service_fee = service_fee - (service_fee * service.discount / 100);
			}
		}

		if let Some(player) = T::StakingPool::is_staking_pool(&account_id) {
				service_fee = service_fee - (service_fee * T::StakingPool::staking_pool_discount() / 100);
		}

		T::OnChargeEVMTxHandler::correct_and_deposit_fee(who, service_fee, already_withdrawn)
	}

	fn pay_priority_fee(tip: U256) {
		T::OnChargeEVMTxHandler::pay_priority_fee(tip)
	}
}
