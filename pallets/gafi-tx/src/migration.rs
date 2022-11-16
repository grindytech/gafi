use crate::{Config, GasPrice};

use frame_support::{
	migration::storage_key_iter,
	pallet_prelude::PhantomData,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Blake2_128Concat,
};
#[cfg(feature = "try-runtime")]
use scale_info::prelude::format;
use sp_core::U256;
use sp_std::vec::Vec;

pub struct GafiTransactionHandler<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for GafiTransactionHandler<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "GafiTransactionHandler", "actually running it");
		let pallet_prefix: &[u8] = b"GafiTx";
		let storage_item_prefix: &[u8] = b"GasPrice";

		let stored_data: Vec<_> = storage_key_iter::<U256, U256, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix,
		)
		.drain()
		.collect();

		let migrated_count = stored_data.len() as u64;

		log::info!(target: "GafiTransactionHandler", "Migrating {:?} elements", migrated_count);

		let gas_price = <T as Config>::GasPrice::get();
		GasPrice::<T>::put(U256::from(gas_price));

		log::info!(target: "GafiTransactionHandler", "almost done");
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
        let gas_price = <T as Config>::GasPrice::get();
		let cur_gas_price = GasPrice::<T>::get();
		assert_eq!(cur_gas_price, gas_price);
		Ok(())
	}
}
