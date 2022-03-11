#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

/*
* tx-handler pallet handle transaction fee
*
*/

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced, WithdrawReasons},
	};
	use pallet_pool::pool::{AuroraZone, PackServiceProvider};
	use pallet_transaction_payment::{OnChargeTransaction};
	use sp_runtime::traits::{DispatchInfoOf, Saturating, Zero};
	use sp_std::marker::PhantomData;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//
	type NegativeImbalanceOf<C, T> =
		<C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

	type Balance<C, T> = <C as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
		type Balance = Balance<C, T>;
		type LiquidityInfo = Option<NegativeImbalanceOf<C, T>>;

		fn withdraw_fee(
			who: &<T>::AccountId,
			_call: &<T>::Call,
			_dispatch_info: &DispatchInfoOf<<T>::Call>,
			fee: Self::Balance,
			tip: Self::Balance,
		) -> Result<Self::LiquidityInfo, frame_support::unsigned::TransactionValidityError> {
			// CurrencyAdapter::<T: Config>::withdraw_fee(who, call, dispatch_info, fee, tip)
			if fee.is_zero() {
				return Ok(None);
			}

			let withdraw_reason = if tip.is_zero() {
				WithdrawReasons::TRANSACTION_PAYMENT
			} else {
				WithdrawReasons::TRANSACTION_PAYMENT | WithdrawReasons::TIP
			};

			let mut service_fee = fee;
			if let Some(player) = T::AuroraZone::is_in_aurora_zone(who) {
				if let Some(service) = T::PackServiceProvider::get_service(player.service) {
					service_fee = fee / service.discount.into();
				}
			}

			match C::withdraw(who, service_fee, withdraw_reason, ExistenceRequirement::KeepAlive) {
				Ok(imbalance) => Ok(Some(imbalance)),
				Err(_) => Err(InvalidTransaction::Payment.into()),
			}
		}

		fn correct_and_deposit_fee(
			who: &<T>::AccountId,
			_dispatch_info: &DispatchInfoOf<<T>::Call>,
			_post_info: &sp_runtime::traits::PostDispatchInfoOf<<T>::Call>,
			corrected_fee: Self::Balance,
			tip: Self::Balance,
			already_withdrawn: Self::LiquidityInfo,
		) -> Result<(), frame_support::unsigned::TransactionValidityError> {
			if let Some(paid) = already_withdrawn {
				// Calculate how much refund we should return
				let refund_amount = paid.peek().saturating_sub(corrected_fee);
				// refund to the the account that paid the fees. If this fails, the
				// account might have dropped below the existential balance. In
				// that case we don't refund anything.
				let refund_imbalance = C::deposit_into_existing(&who, refund_amount)
					.unwrap_or_else(|_| C::PositiveImbalance::zero());
				// merge the imbalance caused by paying the fees and refunding parts of it again.
				let adjusted_paid = paid
					.offset(refund_imbalance)
					.same()
					.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;
				// Call someone else to handle the imbalance (fee and tip separately)
				let (tip, fee) = adjusted_paid.split(tip);
				OU::on_unbalanceds(Some(fee).into_iter().chain(Some(tip)));
			}
			Ok(())
		}
	}

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config + pallet_pool::Config{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type AuroraZone: AuroraZone<Self>;
		type PackServiceProvider: PackServiceProvider<Self>;
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
	impl<T: Config> Pallet<T> {}

	impl<T: Config> Pallet<T> {}
}
