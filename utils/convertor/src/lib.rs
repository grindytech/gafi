#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Currency;
use sp_runtime::DispatchError;

#[derive(Debug)]
pub enum ConvertError {
    TryIntoBalanceFail,
}

pub type BalanceOf<C, A> = <C as Currency<A>>::Balance;

pub fn u128_try_to_balance<C, A>(input: u128) -> Result<BalanceOf<C, A>, DispatchError>
where
    C: Currency<A>,
{
    match input.try_into().ok() {
        Some(val) => Ok(val),
        None => Err(DispatchError::Other("Can not convert u128 to balance")),
    }
}

pub fn u128_to_balance<C, A>(input: u128) -> BalanceOf<C, A>
where
    C: Currency<A>,
{
    input.try_into().ok().unwrap_or_default()
}
