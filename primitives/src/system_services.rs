use crate::constant::ID;
use crate::{pool::Service, ticket::TicketLevel};
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_runtime::{RuntimeDebug, Permill};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct SystemService {
	pub ticket_level: TicketLevel,
	pub service: Service,
	pub value: u128,
}

impl SystemService {
	pub fn new(ticket_level: TicketLevel,tx_limit: u32, discount: Permill, value: u128) -> Self {
		SystemService {
			ticket_level,
			service: Service { tx_limit, discount },
			value,
		}
	}
}

pub trait SystemPool<AccountId> {
	fn join(sender: AccountId, pool_id: ID) -> DispatchResult;
	fn leave(sender: AccountId, pool_id: ID) -> DispatchResult;
	fn get_service(pool_id: ID) -> Option<SystemService>;
}
