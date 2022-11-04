
use crate::{Config, Services};

use frame_support::{
	pallet_prelude::PhantomData,
	traits::{Get, OnRuntimeUpgrade, ReservableCurrency},
	weights::Weight, Blake2_128Concat, Twox64Concat, migration::storage_key_iter,
};
use gafi_primitives::{system_services::{SystemDefaultServices, SystemService}, constant::ID};
#[cfg(feature = "try-runtime")]
use scale_info::prelude::format;
use sp_std::vec::Vec;

pub struct StakingPoolFilter<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for StakingPoolFilter<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "UnitsWithForeignAssetType", "actually running it");
		let pallet_prefix: &[u8] = b"StakingPool";
		let storage_item_prefix: &[u8] = b"Services";

		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<T::AccountId, [(ID, SystemService); 3], Twox64Concat>(
			pallet_prefix,
			storage_item_prefix,
		)
		.drain()
		.collect();

		// let migrated_count = stored_data.len() as u64;

		// log::info!(target: "UnitsWithForeignAssetType", "Migrating {:?} elements", migrated_count);

		// // Write to the new storage with removed and added fields
		// for (asset_id, units) in stored_data {
		// 	// Read the ForeignAssetType for the assetId
		// 	if let Some(asset_type) = AssetIdType::<T>::get(&asset_id) {
		// 		// Populate with ForeignAssetType as key
		// 		AssetTypeUnitsPerSecond::<T>::insert(&asset_type, units)
		// 	}
		// }

		// log::info!(target: "UnitsWithForeignAssetType", "almost done");

		// // Return the weight used. For each migrated mapping there is a read to get it into
		// // memory, a read to get ForeignAssetType and
		// // a write to clear the old stored value, and a write to re-store it.
		// let db_weights = T::DbWeight::get();
		// let rw_count = migrated_count.saturating_mul(2u64);
		// db_weights.reads_writes(rw_count, rw_count)
        0
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {

		Ok(())
	}


	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;
		
		Ok(())
	}
}