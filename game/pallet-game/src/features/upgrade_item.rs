use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Amount, UpgradeItem};
use sp_runtime::Saturating;

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
		level: gafi_support::game::Level,
		fee: BalanceOf<T, I>,
	) -> DispatchResult {
		// ensure collection ownership
		ensure!(
			T::Nfts::is_admin(collection, who),
			Error::<T, I>::NoPermission
		);

		// ensure upgrade level available
		ensure!(
			!UpgradeConfigOf::<T, I>::contains_key((collection, item, level)),
			Error::<T, I>::UpgradeExists,
		);

		if let Some(collection_owner) = T::Nfts::collection_owner(collection) {
			// create item
			T::Nfts::mint_into(collection, new_item, &collection_owner, config, true)?;
		} else {
			return Err(Error::<T, I>::UnknownCollection.into())
		}

		LevelOf::<T, I>::insert(collection, new_item, level);
		OriginItemOf::<T, I>::insert((collection, new_item), (collection, item));

		// insert upgrade config
		UpgradeConfigOf::<T, I>::insert(
			(collection, item, level),
			UpgradeItemConfig {
				item: *new_item,
				fee,
			},
		);

		Self::deposit_event(Event::<T, I>::UpgradeSet {
			who: who.clone(),
			collection: *collection,
			item: *item,
			new_item: *new_item,
			level,
		});

		Ok(())
	}

	fn do_upgrade_item(
		who: &T::AccountId,
		collection: &T::CollectionId,
		item: &T::ItemId,
		amount: Amount,
	) -> DispatchResult {
		ensure!(amount > 0, Error::<T, I>::InvalidAmount);

		let next_level = LevelOf::<T, I>::get(collection, item) + 1;

		// get origin item
		let origin_item = match OriginItemOf::<T, I>::get((collection, item)) {
			Some(val) => val.1,
			None => *item,
		};

		if let Some(config) = UpgradeConfigOf::<T, I>::get((collection, origin_item, next_level)) {
			if let Some(owner) = T::Nfts::collection_owner(collection) {
				<T as pallet::Config<I>>::Currency::transfer(
					&who,
					&owner,
					config.fee.saturating_mul(amount.into()),
					ExistenceRequirement::KeepAlive,
				)?;
			}

			Self::convert_item(who, collection, item, &config.item, amount)?;

			Self::deposit_event(Event::Upgraded {
				who: who.clone(),
				collection: *collection,
				item: *item,
				new_item: config.item,
				amount,
			});

			return Ok(())
		}

		Err(Error::<T, I>::UnknownUpgrade.into())
	}
}
