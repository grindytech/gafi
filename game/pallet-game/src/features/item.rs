use crate::*;
use frame_support::{
	pallet_prelude::*,
};
use gafi_support::game::{Amount, Mutable};

impl<T: Config<I>, I: 'static> Mutable<T::AccountId, T::GameId, T::CollectionId, T::ItemId>
	for Pallet<T, I>
{

	fn mint(
		who: T::AccountId,
		collection_id: T::CollectionId,
		maybe_target: Option<T::AccountId>,
		maybe_amount: Option<Amount>,
	) -> DispatchResult {


		Ok(())
	}

    fn burn(
		who: T::AccountId,
		collection_id: T::CollectionId,
		item_id: T::ItemId,
		maybe_amount: Option<Amount>,
	) -> DispatchResult {

        Ok(())
    }
}