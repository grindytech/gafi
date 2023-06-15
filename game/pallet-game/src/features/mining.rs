use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Bundle, Mining};

impl<T: Config<I>, I: 'static>
	Mining<T::AccountId, BalanceOf<T, I>, T::CollectionId, T::ItemId, T::PoolId> for Pallet<T, I>
{
	fn do_create_dynamic_pool(
		pool: &T::PoolId,
		who: &T::AccountId,
		loot_table: LootTable<T::CollectionId, T::ItemId>,
		fee: BalanceOf<T, I>,
		admin: &T::AccountId,
	) -> DispatchResult {
		// ensure pool is available
		ensure!(
			PoolOf::<T, I>::get(pool).is_none(),
			Error::<T, I>::PoolIdInUse
		);

		// Deposit balance
		<T as Config<I>>::Currency::reserve(&who, T::MiningPoolDeposit::get())?;

		let table =
			LootTableFor::<T, I>::try_from(loot_table.clone()).map_err(|_| Error::<T, I>::ExceedMaxItem)?;
		LootTableOf::<T, I>::insert(pool, table);

		Self::add_total_reserve(pool, Self::total_weight(&loot_table))?;

		// create new pool
		let pool_details = PoolDetails {
			pool_type: PoolType::Dynamic,
			fee,
			owner: who.clone(),
			owner_deposit: T::MiningPoolDeposit::get(),
			admin: admin.clone(),
		};

		// insert storage
		PoolOf::<T, I>::insert(pool, pool_details);

		Ok(())
	}

	fn do_create_stable_pool(
		pool: &T::PoolId,
		who: &T::AccountId,
		loot_table: LootTable<T::CollectionId, T::ItemId>,
		fee: BalanceOf<T, I>,
		admin: &T::AccountId,
	) -> DispatchResult {
		// ensure collection owner & infinite supply
		for fraction in &loot_table {
			if let Some(nft) = &fraction.maybe_nft {
				Self::ensure_collection_owner(who, &nft.collection)?;
				ensure!(
					Self::is_infinite(&nft.collection, &nft.item),
					Error::<T, I>::NotInfiniteSupply
				);
			}
		}

		<T as Config<I>>::Currency::reserve(&who, T::MiningPoolDeposit::get())?;

		// store for random
		{
			LootTableOf::<T, I>::try_mutate(&pool, |reserve_vec| -> DispatchResult {
				reserve_vec
					.try_append(loot_table.clone().into_mut())
					.map_err(|_| <Error<T, I>>::ExceedMaxItem)?;
				Ok(())
			})?;
			Self::add_total_reserve(pool, 100_000)?;
		}

		let pool_details = PoolDetails {
			pool_type: PoolType::Stable,
			fee,
			owner: who.clone(),
			owner_deposit: T::MiningPoolDeposit::get(),
			admin: admin.clone(),
		};

		PoolOf::<T, I>::insert(pool, pool_details);
		Ok(())
	}

	fn do_mint(
		pool: &T::PoolId,
		who: &T::AccountId,
		target: &T::AccountId,
		amount: u32,
	) -> DispatchResult {
		if let Some(pool_details) = PoolOf::<T, I>::get(pool) {
			match pool_details.pool_type {
				PoolType::Dynamic => {
					Self::do_mint_dynamic_pool(pool, who, target, amount)?;
					return Ok(())
				},
				PoolType::Stable => {
					Self::do_mint_stable_pool(pool, who, target, amount)?;
					return Ok(())
				},
			};
		}
		Err(Error::<T, I>::UnknowMiningPool.into())
	}

	fn do_withdraw_pool(pool: &T::PoolId, who: &T::AccountId) -> DispatchResult {
		todo!()
	}

	fn do_mint_dynamic_pool(
		pool: &T::PoolId,
		who: &T::AccountId,
		target: &T::AccountId,
		amount: u32,
	) -> DispatchResult {
		// validating item amount
		{
			let total_item = TotalWeightOf::<T, I>::get(pool);

			ensure!(total_item > 0, Error::<T, I>::SoldOut);
			ensure!(amount <= total_item, Error::<T, I>::ExceedTotalAmount);

			ensure!(
				amount <= T::MaxMintItem::get(),
				Error::<T, I>::ExceedAllowedAmount
			);
		}

		// deposit mining fee
		// if collection owner not found, skip deposit
		if let Some(pool_details) = PoolOf::<T, I>::get(pool) {
			// make a deposit
			<T as pallet::Config<I>>::Currency::transfer(
				&who,
				&pool_details.owner,
				pool_details.fee * amount.into(),
				ExistenceRequirement::KeepAlive,
			)?;

			// random minting
			// let mut items: Vec<T::ItemId> = [].to_vec();
			{
				let mut total_item = TotalWeightOf::<T, I>::get(pool);
				let mut maybe_position = Self::random_number(total_item, Self::gen_random());
				let mut table = LootTableOf::<T, I>::get(pool).clone().into();

				for _ in 0..amount {
					if let Some(position) = maybe_position {
						// ensure position
						ensure!(
							position > 0 && position < total_item,
							Error::<T, I>::MintFailed
						);
						let loot = Self::take_loot(&mut table, position);
						match loot {
							Some(maybe_nft) =>
								if let Some(nft) = maybe_nft {
									Self::repatriate_reserved_item(
										&pool_details.owner,
										&nft.collection,
										&nft.item,
										target,
										1,
										ItemBalanceStatus::Free,
									)?;
								},
							None => return Err(Error::<T, I>::MintFailed.into()),
						};

						total_item = total_item.saturating_sub(1);
						maybe_position = Self::random_number(total_item, position);
					} else {
						return Err(Error::<T, I>::SoldOut.into())
					}
				}

				let table = LootTableFor::<T, I>::try_from(table)
					.map_err(|_| Error::<T, I>::ExceedMaxItem)?;
				LootTableOf::<T, I>::insert(pool, table);
				Self::sub_total_reserve(pool, amount)?;
			}
		}

		// Self::deposit_event(Event::<T, I>::Minted {
		// 	who: who.clone(),
		// 	target: target.clone(),
		// 	collection: *collection,
		// 	items,
		// });
		Ok(())
	}

	fn do_mint_stable_pool(
		pool: &T::PoolId,
		who: &T::AccountId,
		target: &T::AccountId,
		amount: u32,
	) -> DispatchResult {
		// validating item amount
		ensure!(
			amount <= T::MaxMintItem::get(),
			Error::<T, I>::ExceedAllowedAmount
		);

		// deposit mining fee
		// if collection owner not found, skip deposit
		if let Some(pool_details) = PoolOf::<T, I>::get(pool) {
			// make a deposit
			<T as pallet::Config<I>>::Currency::transfer(
				&who,
				&pool_details.owner,
				pool_details.fee * amount.into(),
				ExistenceRequirement::KeepAlive,
			)?;

			// random minting
			// let mut items: Vec<T::ItemId> = [].to_vec();
			{
				let mut total_item = TotalWeightOf::<T, I>::get(pool);
				let mut maybe_position = Self::random_number(total_item, Self::gen_random());
				let reserve = LootTableOf::<T, I>::get(pool);
				for _ in 0..amount {
					if let Some(position) = maybe_position {
						// ensure position
						ensure!(
							position > 0 && position < total_item,
							Error::<T, I>::MintFailed
						);
						let loot = Self::get_loot(&reserve.clone().into(), position);
						match loot {
							Some(maybe_nft) =>
								if let Some(nft) = maybe_nft {
									Self::add_item_balance(target, &nft.collection, &nft.item, 1)?;
								},
							None => return Err(Error::<T, I>::MintFailed.into()),
						};

						maybe_position = Self::random_number(total_item, position);
					} else {
						return Err(Error::<T, I>::SoldOut.into())
					}
				}
			}
		}
		Ok(())
	}
}
