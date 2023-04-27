use crate::*;
use features::*;
use frame_support::pallet_prelude::*;
use gafi_support::common::{BlockNumber};
use sp_runtime::Percent;


impl<T: Config<I>, I: 'static> GameSetting<T::AccountId, T::GameId> for Pallet<T, I> {
    fn do_create_game(
        id: T::GameId,
        owner: T::AccountId,
        maybe_admin: Option<T::AccountId>,
        maybe_name: Option<Vec<u8>>,
    ) -> DispatchResult {

        

        Ok(())
    }

    fn do_set_swap_fee(
        id: T::GameId,
        owner: T::AccountId,
        fee: Percent,
        start_block: BlockNumber,
    ) -> DispatchResult {

        Ok(())
    }
}
