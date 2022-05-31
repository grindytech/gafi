#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::traits::{ExistenceRequirement, Currency};
use sp_runtime::DispatchResult;
use frame_support::traits::fungible::{Inspect, Transfer};

pub fn transfer_all<T, C>(from: &T::AccountId, to: &T::AccountId, keep_alive: bool) -> DispatchResult
where
    T: pallet_balances::Config,
    C: Currency<T::AccountId>,
{
    let reducible_balance: u128 =
        pallet_balances::pallet::Pallet::<T>::reducible_balance(&from, keep_alive)
            .try_into()
            .ok()
            .unwrap();
    let existence = if keep_alive {
        ExistenceRequirement::KeepAlive
    } else {
        ExistenceRequirement::AllowDeath
    };
    C::transfer(
        &from,
        &to,
        reducible_balance.try_into().ok().unwrap(),
        existence,
    )
}
