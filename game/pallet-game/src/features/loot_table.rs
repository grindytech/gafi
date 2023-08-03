use crate::*;
use gafi_support::game::{LootTable, NFT};
use sp_runtime::Saturating;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Retrieves loot from the provided `table` based on the given `random` position.
	/// The function uses the `random` value to select a random position within the LootTable
	/// and find the corresponding loot based on item weights.
	///
	/// # Arguments
	///
	/// * `table` - A reference to the LootTable containing items with associated weights and loot
	///   data.
	/// * `random` - A random number used to determine the loot position.
	///
	/// # Returns
	///
	/// * `Some(NFT)` - If an item is found at the determined position and its weight is greater
	///   than 0, it returns the associated NFT (Non-Fungible Token).
	/// * `None` - If no item is found at the determined position or its weight is 0, it returns
	///   None.
	pub(crate) fn get_loot(
		table: &LootTable<T::CollectionId, T::ItemId>,
		random: u32,
	) -> Option<Option<NFT<T::CollectionId, T::ItemId>>> {
		let mut positon = random;
		let mut i = 0;

		while i < table.len() && positon > table[i].weight {
			positon -= table[i].weight;
			i += 1;
		}

		if i < table.len() {
			Some(table[i].clone().maybe_nft)
		} else {
			None
		}
	}

	/// Takes loot from the provided `table` based on the given `random` position.
	/// The function iterates through the `table`, deducts the weight of the selected item,
	/// and returns the corresponding loot's `maybe_nft` if found.
	///
	/// If the `random` position is out of range or no item is found at that position,
	/// the function returns `None`.
	///
	/// # Arguments
	///
	/// * `table` - A mutable reference to the LootTable containing items with associated weights
	///   and loot data.
	/// * `random` - A random number used to determine the loot position.
	///
	/// # Returns
	///
	/// * `Some(NFT)` - If an item is found at the determined position and its weight is greater
	///   than 0, it returns the associated NFT (Non-Fungible Token) after deducting its weight.
	/// * `None` - If the `random` position is out of range or no item is found at that position, or
	///   if the weight of the selected item is 0, it returns None.
	pub(crate) fn take_loot(
		table: &mut LootTable<T::CollectionId, T::ItemId>,
		random: u32,
	) -> Option<Option<NFT<T::CollectionId, T::ItemId>>> {
		let mut positon = random;
		let mut i = 0;

		while i < table.len() && positon > table[i].weight {
			positon -= table[i].weight;
			i += 1;
		}

		if i < table.len() {
			table[i].weight.saturating_dec();
			Some(table[i].clone().maybe_nft)
		} else {
			None
		}
	}
}

#[cfg(test)]
#[test]
fn get_loot_should_works() {
	use gafi_support::game::Loot;

	use crate::mock::{new_test_ext, run_to_block, PalletGame};

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
		]
		.to_vec();
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 1).unwrap(),
			table[0].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 200).unwrap(),
			table[0].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 201).unwrap(),
			table[1].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 400).unwrap(),
			table[1].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 401).unwrap(),
			table[2].maybe_nft
		);
		assert_eq!(
			PalletGame::get_loot(&table.clone(), 600).unwrap(),
			table[2].maybe_nft
		);
		assert_eq!(PalletGame::get_loot(&table.clone(), 601), None);
	})
}

#[cfg(test)]
#[test]
fn take_loot_should_works() {
	use gafi_support::game::Loot;

	use crate::mock::{new_test_ext, run_to_block, PalletGame};

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
			PalletGame::take_loot(&mut table, 1).unwrap(),
			table[0].clone().maybe_nft
		);
		assert_eq!(
			PalletGame::take_loot(&mut table, 199).unwrap(),
			table[0].clone().maybe_nft
		);
		assert_eq!(
			PalletGame::take_loot(&mut table, 199).unwrap(),
			table[1].clone().maybe_nft
		);
		assert_eq!(
			PalletGame::take_loot(&mut table, 397).unwrap(),
			table[1].clone().maybe_nft
		);
		assert_eq!(
			PalletGame::take_loot(&mut table, 397).unwrap(),
			table[2].clone().maybe_nft
		);
		assert_eq!(
			PalletGame::take_loot(&mut table, 595).unwrap(),
			table[2].clone().maybe_nft
		);
		assert_eq!(PalletGame::take_loot(&mut table, 595), None);
	})
}
