#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, Get, Imbalance, OnUnbalanced},
	};
	use pallet_transaction_payment::{CurrencyAdapter, OnChargeTransaction};
	use sp_runtime::traits::{ DispatchInfoOf};
	use sp_std::{marker::PhantomData};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//

	type NegativeImbalanceOf<C, T> =
		<C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
		type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub struct AurCurrencyAdapter<C, OU>(PhantomData<(C, OU)>);

	impl<T, C, OU> OnChargeTransaction<T> for AurCurrencyAdapter<C, OU>
	where
		T: Config,
		T::TransactionByteFee:
			Get<<C as Currency<<T as frame_system::Config>::AccountId>>::Balance>,
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
		type Balance =  BalanceOf<T>;
		type LiquidityInfo = Option<NegativeImbalanceOf<C, T>>;

		fn withdraw_fee(
			who: &<T>::AccountId,
			call: &<T>::Call,
			dispatch_info: &DispatchInfoOf<<T>::Call>,
			fee: Self::Balance,
			tip: Self::Balance,
		) -> Result<Self::LiquidityInfo, frame_support::unsigned::TransactionValidityError> {
			todo!()
		}

		fn correct_and_deposit_fee(
			who: &<T>::AccountId,
			dispatch_info: &DispatchInfoOf<<T>::Call>,
			post_info: &sp_runtime::traits::PostDispatchInfoOf<<T>::Call>,
			corrected_fee: Self::Balance,
			tip: Self::Balance,
			already_withdrawn: Self::LiquidityInfo,
		) -> Result<(), frame_support::unsigned::TransactionValidityError> {
			todo!()
		}
	}

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// type EVM: pallet_evm::EvmConfig;

		type Currency: Currency<Self::AccountId>;
	}

	// Errors.
	#[derive(PartialEq)]
	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	// Storage

	#[pallet::call]
	impl<T: Config> Pallet<T> {
	}

	impl<T: Config> Pallet<T> {}
}
