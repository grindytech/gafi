use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::common::types::BlockNumber;


impl<T: Config<I>, I: 'static> GameSetting<Error<T>, T::AccountId, T::GameId> for Pallet<T, I> {
    fn create_game(
        id: T::GameId,
        owner: T::AccountId,
        admin: Option<T::AccountId>,
        name: Vec<u8>,
    ) -> Result<T::GameId, Error<T>> {
        let bounded_name: BoundedVec<_, _> =
            name.try_into().map_err(|_| Error::<T>::NameTooLong)?;
        ensure!(
            bounded_name.len() >= T::MinNameLength::get() as usize,
            Error::<T>::NameTooShort
        );

        Games::<T, I>::insert(id, (owner, bounded_name));
        Ok(id)
    }

    fn set_swapping_fee(
        id: T::GameId,
        fee: u8,
        start_block: BlockNumber,
    ) -> Result<(), Error<T>> {
        ensure!(fee <= T::MaxSwapFee::get(), Error::SwapFeeTooHigh);
        SwapFee::<T, I>::insert(id, (fee, start_block));
        Ok(())
    }
}
