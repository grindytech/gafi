use crate::{types::Item, *};
use frame_support::{log, pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, MutateItem, UpgradeItem, Metadata};

impl<T: Config<I>, I: 'static> UpgradeItem<T::AccountId, BalanceOf<T, I>, T::CollectionId, T::ItemId, T::StringLimit>
	for Pallet<T, I>
{
    fn do_set_upgrade_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		data: &Metadata<T::StringLimit>,
		level: gafi_support::game::Level,
		fee: BalanceOf<T, I>,
	) -> DispatchResult {
        todo!()
    }

    fn do_upgrade_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> DispatchResult {
        todo!()
    }
}