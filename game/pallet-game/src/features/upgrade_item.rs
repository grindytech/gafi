use core::ops::Mul;

use crate::{types::Item, *};
use frame_support::{
	log::{self},
	pallet_prelude::*,
	traits::ExistenceRequirement,
};
use gafi_support::game::{Amount, Level, Metadata, MutateItem, UpgradeItem};

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

		LevelOf::<T, I>::insert(collection, new_item, level);
		OriginItemOf::<T, I>::insert((collection, new_item), (collection, item));

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
		let next_level = LevelOf::<T, I>::get(collection, item) + 1;
		
		// get origin item
		let origin_item = match OriginItemOf::<T, I>::get((collection, item)) {
			Some(val) => val.1,
			None => *item,
		};

		if let Some(config) = UpgradeConfigOf::<T, I>::get((collection, origin_item, next_level)) {
				Self::minus_item_balance(who, collection, item, amount)?;
				Self::add_item_balance(who, collection, &config.item, amount)?;
		}

		Ok(())
	}
}
