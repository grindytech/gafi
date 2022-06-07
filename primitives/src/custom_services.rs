use crate::constant::ID;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::{RuntimeDebug, Permill};
use sp_std::vec::Vec;
use crate::pool::{Service};

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct CustomService<AccountId> {
	pub service: Service,
	pub sponsor: AccountId,
	pub targets: Vec<H160>,
}

impl<AccountId> CustomService<AccountId> {
	pub fn new(targets: Vec<H160>, tx_limit: u32, discount: Permill, sponsor: AccountId) -> Self {
		CustomService {
			targets,
			service: Service { tx_limit, discount },
			sponsor,
		}
	}
}

pub trait CustomPool<AccountId> {
	fn join(sender: AccountId, pool_id: ID) -> DispatchResult;
	fn leave(sender: AccountId) -> DispatchResult;
	fn get_service(pool_id: ID) -> Option<CustomService<AccountId>>;

	#[cfg(feature = "runtime-benchmarks")]
	fn add_default(owner: AccountId, pool_id: ID);
}
