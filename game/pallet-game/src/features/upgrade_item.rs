use core::ops::Mul;

use crate::{types::Item, *};
use frame_support::{log, pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, Metadata, MutateItem, UpgradeItem};

impl<T: Config<I>, I: 'static>
	UpgradeItem<
		T::AccountId,
		BalanceOf<T, I>,
		T::CollectionId,
		T::ItemId,
		ItemConfig,
		T::StringLimit,
	> for Pallet<T, I>
{
	fn do_set_upgrade_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
        new_item: &T::ItemId,
		config: &ItemConfig,
		data: Metadata<T::StringLimit>,
		level: gafi_support::game::Level,
		fee: BalanceOf<T, I>,
	) -> DispatchResult {
		// ensure collection ownership
		ensure!(
			T::Nfts::collection_owner(collection) == Some(who.clone()),
			Error::<T, I>::NoPermission
		);

		// ensure upgrade level available
		ensure!(
			!UpgradeConfigOf::<T, I>::contains_key((collection, item, level)),
			Error::<T, I>::UpgradeExists,
		);

		// create item
		let _ = T::Nfts::mint_into(collection, new_item, who, config, false)?;

		// insert upgrade config
		UpgradeConfigOf::<T, I>::insert(
			(collection, item, level),
			ItemUpgradeConfig {
				collection: *collection,
				origin: *item,
				item: *new_item,
				level,
				fee,
				data,
			},
		);

		Ok(())
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
