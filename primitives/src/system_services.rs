use crate::{pool::Service, ticket::TicketLevel};
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct SystemService {
	pub service: Service,
	pub value: u128,
}

impl SystemService {
	pub fn new(tx_limit: u32, discount: u8, value: u128) -> Self {
		SystemService {
			service: Service { tx_limit, discount },
			value,
		}
	}
}

pub trait SystemPool<AccountId> {
	fn join(sender: AccountId, level: TicketLevel) -> DispatchResult;
	fn leave(sender: AccountId) -> DispatchResult;
	fn get_service(level: TicketLevel) -> Option<SystemService>;
}
