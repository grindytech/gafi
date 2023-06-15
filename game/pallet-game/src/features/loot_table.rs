use crate::*;
use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
use gafi_support::game::{Bundle, LootTable, Mining, NFT};

impl<T: Config<I>, I: 'static> Pallet<T, I> {

    /// Get Basic Loot Mechanism
    /// 
    /// Returns an NFT depending on the `position`.
    /// 
    ///- `table`: loot table
    ///- `position`: rolling for loot, position must be in 0..`total weight - 1`
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
		];
		assert_eq!(
			PalletGame::get_loot(&table.clone().to_vec(), 0).unwrap(),
			table[0].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone().to_vec(), 199).unwrap(),
			table[0].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone().to_vec(), 200).unwrap(),
			table[1].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone().to_vec(), 399).unwrap(),
			table[1].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone().to_vec(), 400).unwrap(),
			table[2].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone().to_vec(), 599).unwrap(),
			table[2].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone().to_vec(), 600),
			None
		);
	})
}
