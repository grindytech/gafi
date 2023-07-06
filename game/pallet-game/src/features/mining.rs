use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement, StorageNMap};
use gafi_support::game::{Mining, MintSettings, NFT, MintType};

impl<T: Config<I>, I: 'static>
	Mining<T::AccountId, BalanceOf<T, I>, T::CollectionId, T::ItemId, T::PoolId, T::BlockNumber>
	for Pallet<T, I>
{
	fn do_create_dynamic_pool(
		pool: &T::PoolId,
		who: &T::AccountId,
		loot_table: LootTable<T::CollectionId, T::ItemId>,
		admin: &T::AccountId,
		mint_settings: MintSettings<BalanceOf<T, I>, T::BlockNumber, T::CollectionId>,
	) -> DispatchResult {
		// ensure pool is available
		ensure!(
			PoolOf::<T, I>::get(pool).is_none(),
			Error::<T, I>::PoolIdInUse
		);

		// Deposit balance
		<T as Config<I>>::Currency::reserve(&who, T::MiningPoolDeposit::get())?;

		// reserve resource
		for loot in &loot_table {
			if let Some(nft) = &loot.maybe_nft {
				Self::reserved_item(who, &nft.collection, &nft.item, loot.weight)?;
			}
		}

		let table = LootTableFor::<T, I>::try_from(loot_table.clone())
			.map_err(|_| Error::<T, I>::ExceedMaxLoot)?;
		LootTableOf::<T, I>::insert(pool, table);

		// create new pool
		let pool_details = PoolDetails {
			pool_type: PoolType::Dynamic,
			owner: who.clone(),
			owner_deposit: T::MiningPoolDeposit::get(),
			admin: admin.clone(),
			mint_settings,
		};

		// insert storage
		PoolOf::<T, I>::insert(pool, pool_details);
		Self::deposit_event(Event::<T, I>::MiningPoolCreated {
			pool: *pool,
			who: who.clone(),
			pool_type: PoolType::Dynamic,
			table: loot_table,
		});

		Ok(())
	}

	fn do_create_stable_pool(
		pool: &T::PoolId,
		who: &T::AccountId,
		loot_table: LootTable<T::CollectionId, T::ItemId>,
		admin: &T::AccountId,
		mint_settings: MintSettings<BalanceOf<T, I>, T::BlockNumber, T::CollectionId>,
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
		let table = LootTableFor::<T, I>::try_from(loot_table.clone())
			.map_err(|_| Error::<T, I>::ExceedMaxLoot)?;

		LootTableOf::<T, I>::insert(pool, table);

		let pool_details = PoolDetails {
			pool_type: PoolType::Stable,
			owner: who.clone(),
			owner_deposit: T::MiningPoolDeposit::get(),
			admin: admin.clone(),
			mint_settings,
		};

		PoolOf::<T, I>::insert(pool, pool_details);
		Self::deposit_event(Event::<T, I>::MiningPoolCreated {
			pool: *pool,
			who: who.clone(),
			pool_type: PoolType::Stable,
			table: loot_table,
		});
		Ok(())
	}

	fn do_mint(
		pool: &T::PoolId,
		who: &T::AccountId,
		target: &T::AccountId,
		amount: u32,
	) -> DispatchResult {
		if let Some(pool_details) = PoolOf::<T, I>::get(pool) {
			// verify mint settings
			let mint_settings = pool_details.mint_settings;
			let block_number = <frame_system::Pallet<T>>::block_number();
			if let Some(start_block) = mint_settings.start_block {
				ensure!(block_number >= start_block, Error::<T, I>::MintNotStarted);
			}
			if let Some(end_block) = mint_settings.end_block {
				ensure!(block_number <= end_block, Error::<T, I>::MintEnded);
			}
			match mint_settings.mint_type {
				MintType::HolderOf(collection) => {
					ensure!(
						ItemBalanceOf::<T, I>::contains_prefix((who.clone(), collection,)),
						Error::<T, I>::NotWhitelisted
					);
				},
				_ => {},
			};

			match pool_details.pool_type {
				PoolType::Dynamic => {
					return Self::do_mint_dynamic_pool(pool, who, target, amount)
				},
				PoolType::Stable => {
					return Self::do_mint_stable_pool(pool, who, target, amount)
				},
			}
		}
		Err(Error::<T, I>::UnknownMiningPool.into())
	}

	fn do_mint_dynamic_pool(
		pool: &T::PoolId,
		who: &T::AccountId,
		target: &T::AccountId,
		amount: u32,
	) -> DispatchResult {
		// validating item amount
		let mut table = LootTableOf::<T, I>::get(pool).clone().into();
		{
			let total_weight = Self::total_weight(&table);
			ensure!(total_weight > 0, Error::<T, I>::SoldOut);
			ensure!(amount <= total_weight, Error::<T, I>::ExceedTotalAmount);
			ensure!(
				amount <= T::MaxMintItem::get(),
				Error::<T, I>::ExceedAllowedAmount
			);
		}

		if let Some(pool_details) = PoolOf::<T, I>::get(pool) {
			// make a deposit
			<T as pallet::Config<I>>::Currency::transfer(
				&who,
				&pool_details.owner,
				pool_details.mint_settings.price * amount.into(),
				ExistenceRequirement::KeepAlive,
			)?;

			// random minting
			let mut nfts: Vec<NFT<T::CollectionId, T::ItemId>> = Vec::new();
			{
				let mut total_weight = Self::total_weight(&table);
				let mut maybe_position = Self::random_number(total_weight, Self::gen_random());
				for _ in 0..amount {
					if let Some(position) = maybe_position {
						// ensure position
						ensure!(position < total_weight, Error::<T, I>::MintFailed);
						match Self::take_loot(&mut table, position) {
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
									nfts.push(nft);
								},
							None => return Err(Error::<T, I>::MintFailed.into()),
						};

						total_weight = total_weight.saturating_sub(1);
						maybe_position = Self::random_number(total_weight, position);
					} else {
						return Err(Error::<T, I>::SoldOut.into())
					}
				}

				let table = LootTableFor::<T, I>::try_from(table)
					.map_err(|_| Error::<T, I>::ExceedMaxLoot)?;
				LootTableOf::<T, I>::insert(pool, table);
				Self::deposit_event(Event::<T, I>::Minted {
					pool: *pool,
					who: who.clone(),
					target: target.clone(),
					nfts,
				});
				return Ok(())
			}
		}
		Err(Error::<T, I>::UnknownMiningPool.into())
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

		if let Some(pool_details) = PoolOf::<T, I>::get(pool) {
			// make a deposit
			<T as pallet::Config<I>>::Currency::transfer(
				&who,
				&pool_details.owner,
				pool_details.mint_settings.price * amount.into(),
				ExistenceRequirement::KeepAlive,
			)?;

			// random minting
			let mut nfts: Vec<NFT<T::CollectionId, T::ItemId>> = Vec::new();
			{
				let table = LootTableOf::<T, I>::get(pool).into();
				let total_weight = Self::total_weight(&table);
				let mut maybe_position = Self::random_number(total_weight, Self::gen_random());

				for _ in 0..amount {
					if let Some(position) = maybe_position {
						// ensure position
						ensure!(position < total_weight, Error::<T, I>::MintFailed);
						match Self::get_loot(&table, position) {
							Some(maybe_nft) =>
								if let Some(nft) = maybe_nft {
									Self::add_item_balance(target, &nft.collection, &nft.item, 1)?;
									nfts.push(nft);
								},
							None => return Err(Error::<T, I>::MintFailed.into()),
						};

						maybe_position = Self::random_number(total_weight, position);
					} else {
						return Err(Error::<T, I>::SoldOut.into())
					}
				}
			}

			Self::deposit_event(Event::<T, I>::Minted {
				pool: *pool,
				who: who.clone(),
				target: target.clone(),
				nfts,
			});
			return Ok(())
		}
		Err(Error::<T, I>::UnknownMiningPool.into())
	}
}
