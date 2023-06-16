use crate::*;
use gafi_support::game::{LootTable, NFT};

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Get Basic Loot Mechanism
	///
	/// Returns an NFT depending on the `position`.
	///
	/// - `table`: loot table
	/// - `position`: rolling for loot, position must be in 0..`total weight - 1`
	pub(crate) fn get_loot(
		table: &LootTable<T::CollectionId, T::ItemId>,
		position: u32,
	) -> Option<Option<NFT<T::CollectionId, T::ItemId>>> {
		let mut sum: u32 = 0_u32;
		for item in table {
			if item.weight == 0 {
				continue
			};

			if (item.weight + sum - 1) >= position {
				return Some(item.clone().maybe_nft)
			} else {
				sum += item.weight;
			}
		}
		return None
	}

	/// Get Basic Loot Mechanism
	///
	/// Take and return an NFT depending on the `position`.
	///
	/// - `table`: loot table
	/// - `position`: rolling for loot, position must be in 0..`total weight - 1`
	pub(crate) fn take_loot(
		table: &mut LootTable<T::CollectionId, T::ItemId>,
		position: u32,
	) -> Option<Option<NFT<T::CollectionId, T::ItemId>>> {
		let mut sum: u32 = 0_u32;
		for mut item in table {
			if item.weight == 0 {
				continue
			};

			if (item.weight + sum - 1) >= position {
				item.weight -= 1;
				return Some(item.clone().maybe_nft)
			} else {
				sum += item.weight;
			}
		}
		return None
	}
}

#[cfg(test)]
#[test]
fn get_loot_should_works() {
	use frame_support::assert_err;
	use gafi_support::game::Loot;

	use crate::mock::{new_test_ext, run_to_block, PalletGame, Test};

	new_test_ext().execute_with(|| {
		run_to_block(2);
		let table = [
			Loot {
				maybe_nft: Some(NFT {
					collection: 0,
					item: 0,
				}),
				weight: 200,
			},
			Loot {
				maybe_nft: Some(NFT {
					collection: 1,
					item: 1,
				}),
				weight: 200,
			},
			Loot {
				maybe_nft: Some(NFT {
					collection: 2,
					item: 2,
				}),
				weight: 200,
			},
		].to_vec();
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 0).unwrap(),
			table[0].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 199).unwrap(),
			table[0].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 200).unwrap(),
			table[1].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 399).unwrap(),
			table[1].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 400).unwrap(),
			table[2].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 599).unwrap(),
			table[2].maybe_nft
		);
		assert_eq!(PalletGame::get_loot(&table.clone(), 600), None);
	})
}

#[cfg(test)]
#[test]
fn take_loot_should_works() {
	use frame_support::assert_err;
	use gafi_support::game::Loot;

	use crate::mock::{new_test_ext, run_to_block, PalletGame, Test};

	new_test_ext().execute_with(|| {
		run_to_block(2);
		let mut table = [
			Loot {
				maybe_nft: Some(NFT {
					collection: 0,
					item: 0,
				}),
				weight: 200,
			},
			Loot {
				maybe_nft: Some(NFT {
					collection: 1,
					item: 1,
				}),
				weight: 200,
			},
			Loot {
				maybe_nft: Some(NFT {
					collection: 2,
					item: 2,
				}),
				weight: 200,
			},
		]
		.to_vec();
		assert_eq!(
			PalletGame::take_loot(&mut table, 0).unwrap(),
			table[0].clone().maybe_nft
		);
		assert_eq!(
			PalletGame::take_loot(&mut table, 199).unwrap(),
			table[1].clone().maybe_nft
		);
		assert_eq!(
			PalletGame::take_loot(&mut table, 398).unwrap(),
			table[2].clone().maybe_nft
		);
		assert_eq!(
			PalletGame::take_loot(&mut table, 596).unwrap(),
			table[2].clone().maybe_nft
		);
		assert_eq!(PalletGame::take_loot(&mut table, 596), None);
	})
}
