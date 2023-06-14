use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Bundle, Mining};

impl<T: Config<I>, I: 'static>
	Mining<T::AccountId, BalanceOf<T, I>, T::CollectionId, T::ItemId, T::PoolId> for Pallet<T, I>
{
	fn do_create_dynamic_pool(
		pool: &T::PoolId,
		who: &T::AccountId,
		resource: Bundle<T::CollectionId, T::ItemId>,
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

		// Reserve item balance
		Self::reserved_bundle(who, resource.clone())?;
		ReserveOf::<T, I>::try_mutate(&pool, |reserve_vec| -> DispatchResult {
			reserve_vec
				.try_append(resource.clone().into_mut())
				.map_err(|_| <Error<T, I>>::ExceedMaxItem)?;
			Ok(())
		})?;
		Self::add_total_reserve(pool, Self::count_amount(&resource))?;

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
		dist: Bundle<T::CollectionId, T::ItemId>,
		fee: BalanceOf<T, I>,
		admin: &T::AccountId,
	) -> DispatchResult {
		// ensure collection owner & infinite supply
		for fraction in &dist {
			Self::ensure_collection_owner(who, &fraction.collection)?;
			ensure!(
				Self::is_infinite(&fraction.collection, &fraction.item),
				Error::<T, I>::NotInfiniteSupply
			);
		}

		<T as Config<I>>::Currency::reserve(&who, T::MiningPoolDeposit::get())?;

		// store for random
		{
			ReserveOf::<T, I>::try_mutate(&pool, |reserve_vec| -> DispatchResult {
				reserve_vec
					.try_append(dist.clone().into_mut())
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
			let total_item = TotalReserveOf::<T, I>::get(pool);

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
				let mut total_item = TotalReserveOf::<T, I>::get(pool);
				let mut maybe_position = Self::random_number(total_item, Self::gen_random());
				for _ in 0..amount {
					if let Some(position) = maybe_position {
						match Self::withdraw_reserve(pool, position) {
							Ok(package) => {
								Self::repatriate_reserved_item(
									&pool_details.owner,
									&package.collection,
									&package.item,
									target,
									1,
									ItemBalanceStatus::Free,
								)?;
								// items.push(item);
							},
							Err(err) => return Err(err.into()),
						};
						total_item = total_item.saturating_sub(1);
						maybe_position = Self::random_number(total_item, position);
					} else {
						return Err(Error::<T, I>::SoldOut.into())
					}
				}
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
				let mut total_item = TotalReserveOf::<T, I>::get(pool);
				let mut maybe_position = Self::random_number(total_item, Self::gen_random());
				let reserve = ReserveOf::<T, I>::get(pool);
				for _ in 0..amount {
					if let Some(position) = maybe_position {
						match Self::get_reserve(&reserve, position) {
							Ok(package) => {
								Self::add_item_balance(
									target,
									&package.collection,
									&package.item,
									1,
								)?;
								// items.push(item);
							},
							Err(err) => return Err(err.into()),
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
