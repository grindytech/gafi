use crate::{Config, Services};

use frame_support::{
	migration::storage_key_iter,
	pallet_prelude::PhantomData,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Blake2_128Concat,
};
use gafi_primitives::{
	common::constant::ID,
	pool::system_services::{SystemDefaultServices, SystemService},
};
#[cfg(feature = "try-runtime")]
use scale_info::prelude::format;
use sp_std::vec::Vec;

pub struct UpfrontPoolFilter<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for UpfrontPoolFilter<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "UpfrontPoolFilter", "actually running it");
		let pallet_prefix: &[u8] = b"UpfrontPool";
		let storage_item_prefix: &[u8] = b"Services";

		let stored_data: Vec<_> = storage_key_iter::<ID, SystemService, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix,
		)
		.drain()
		.collect();

		let migrated_count = stored_data.len() as u64;

		log::info!(target: "UpfrontPoolFilter", "Migrating {:?} elements", migrated_count);
		// clean old data
		let _ = Services::<T>::clear(0u32, None);
		
		let services = <T as Config>::UpfrontServices::get_default_services();
		for service in services.data {
			Services::<T>::insert(service.0, service.1);
		}

		log::info!(target: "UpfrontPoolFilter", "almost done");
		let db_weights = T::DbWeight::get();
		let rw_count = migrated_count.saturating_mul(2u64);
		db_weights.reads_writes(rw_count, rw_count)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		let services = <T as Config>::UpfrontServices::get_default_services();
		for service in services.data {
			let pool_service = Services::<T>::get(service.0).unwrap();
			assert_eq!(pool_service, service.1);
		}
		Ok(())
	}
}
