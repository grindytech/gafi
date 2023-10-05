#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::weights::WeightInfo;
use frame_support::traits::{Currency, ExistenceRequirement, BuildGenesisConfig};
use gafi_support::pallet::cache::Cache;
use gu_convertor::{balance_try_to_u128, u128_to_balance};
pub use pallet::*;
use sp_std::{vec, vec::Vec};

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
	use frame_support::{pallet_prelude::*, BoundedVec};
	use frame_system::pallet_prelude::*;

	type MaxFundingAccount = ConstU32<3>;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type AccountOf<T> = <T as frame_system::Config>::AccountId;
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism.
		type Currency: Currency<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Add Cache
		type Cache: Cache<Self::AccountId, AccountOf<Self>, u128>;

		/// Faucet Amount
		type FaucetAmount: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Holding all the accounts
	#[pallet::storage]
	pub(super) type GenesisAccounts<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, MaxFundingAccount>, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub genesis_accounts: Vec<T::AccountId>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			for i in 0..self.genesis_accounts.len() {
				let _ = <GenesisAccounts<T>>::try_append(self.genesis_accounts[i].clone())
					.map_or((), |_| {});
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Transferred(T::AccountId, T::AccountId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoGenesisAccountAvailable,
		NotEnoughBalance,
		DontBeGreedy,
		PleaseWait,
		OutOfFaucet,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// faucet
		///
		/// The origin must be Signed
		///
		/// Weight: `O(1)`
		#[pallet::call_index(0)]
		#[pallet::weight((
			0,
			DispatchClass::Normal,
			Pays::No
		))]
		pub fn faucet(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let genesis_accounts = GenesisAccounts::<T>::get();
			let faucet_amount = T::FaucetAmount::get();
			let faucet_u128 = balance_try_to_u128::<<T as pallet::Config>::Currency, T::AccountId>(
				faucet_amount,
			)?;

			let amount = faucet_u128.saturating_div(10u128);
			let faucet = u128_to_balance::<<T as pallet::Config>::Currency, T::AccountId>(amount);

			ensure!(Self::get_cache(&sender) == None, <Error<T>>::PleaseWait);

			ensure!(
				T::Currency::free_balance(&sender) < faucet,
				<Error<T>>::DontBeGreedy
			);

			for account in genesis_accounts {
				match T::Currency::transfer(
					&account,
					&sender,
					faucet_amount,
					ExistenceRequirement::KeepAlive,
				) {
					Ok(_) => {
						Self::insert_cache(sender, faucet_amount);
						return Ok(())
					},
					Err(_) => continue,
				}
			}
			Err(Error::<T>::OutOfFaucet.into())
		}

		/// donate
		///
		/// The origin must be Signed
		///
		/// Parameters:
		/// - `amount`: donation amount
		///
		/// Weight: `O(1)`
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::donate(50u32))]
		pub fn donate(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let from = ensure_signed(origin)?;

			ensure!(
				T::Currency::free_balance(&from) > amount,
				<Error<T>>::NotEnoughBalance
			);

			let genesis_accounts = GenesisAccounts::<T>::get();
			if let Some(genesis_account) = genesis_accounts.first() {
				T::Currency::transfer(
					&from,
					genesis_account,
					amount,
					ExistenceRequirement::KeepAlive,
				)?;

				Self::deposit_event(Event::Transferred(from, genesis_account.clone(), amount));
			} else {
				return Err(<Error<T>>::NoGenesisAccountAvailable.into())
			}
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn insert_cache(sender: T::AccountId, faucet_amount: BalanceOf<T>) -> Option<()> {
			match faucet_amount.try_into() {
				Ok(value) => Some(T::Cache::insert(&sender, sender.clone(), value)),
				Err(_) => None,
			}
		}

		fn get_cache(sender: &T::AccountId) -> Option<u128> {
			if let Some(faucet_cache) = T::Cache::get(sender, sender.clone()) {
				return Some(faucet_cache)
			}
			None
		}
	}
}
