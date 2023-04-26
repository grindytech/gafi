use crate::*;
use frame_support::pallet_prelude::*;
use gafi_support::common::types::BlockNumber;
use sp_runtime::Percent;


impl<T: Config<I>, I: 'static> GameSetting<T::AccountId, T::GameId> for Pallet<T, I> {
    fn create_game(
        id: T::GameId,
        owner: T::AccountId,
        admin: Option<T::AccountId>,
        name: Vec<u8>,
    ) -> DispatchResult {

        Ok(())
    }

    fn set_swap_fee(
        id: T::GameId,
        owner: T::AccountId,
        fee: Percent,
        start_block: BlockNumber,
    ) -> DispatchResult {

        Ok(())
    }
}
