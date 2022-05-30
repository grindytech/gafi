#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, Imbalance, OnUnbalanced},
};
use frame_system::Config;

#[derive(Debug)]
pub enum ConvertError {
    TryIntoBalanceFail,
}

pub type BalanceOf<C, A> =
    <C as Currency<A>>::Balance;

// pub fn u128_try_to_balance<BalanceOf>(input: u128) -> Result<BalanceOf, ConvertError>
// where
//     BalanceOf: std::convert::From<u128>,
// {
//     match input.try_into().ok() {
//         Some(val) => Ok(val),
//         None => Err(ConvertError::TryIntoBalanceFail),
//     }
// }

pub fn u128_to_balance<C, A>(input: u128) -> BalanceOf<C, A>

where
C:  Currency<A>,
{
    input.try_into().ok().unwrap_or_default()
}
