use crate::constant::ID;
use crate::ticket::Ticket;
use crate::pool::Service;
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
	pub id: ID,
	pub service: Service,
	pub value: u128,
}

impl SystemService {
	pub fn new(id: ID,tx_limit: u32, discount: Permill, value: u128) -> Self {
		SystemService {
			id,
			service: Service { tx_limit, discount },
			value,
		}
	}
}

pub trait SystemPool<AccountId> {
	fn join(sender: AccountId, pool_id: ID) -> DispatchResult;
	fn leave(sender: AccountId) -> DispatchResult;
	fn get_service(pool_id: ID) -> Option<SystemService>;
	fn get_ticket(sender: AccountId) -> Option<Ticket<AccountId>>;
}

impl<AccountId> SystemPool<AccountId> for () {
	fn join(_sender: AccountId, _pool_id: ID) -> DispatchResult {
		Ok(())
	}
	fn leave(_sender: AccountId) -> DispatchResult {
		Ok(())
	}
	fn get_service(_pool_id: ID) -> Option<SystemService> {
		Default::default()
	}
	fn get_ticket(_sender: AccountId) -> Option<Ticket<AccountId>> {
		Default::default()
	}
}

pub trait SystemDefaultServices {
	fn get_default_services() -> [(ID, SystemService); 3];
}

// pub struct Convertor;

// impl Convertor {
// 	pub fn into_id(ticket: SystemTicket) -> ID {
// 		ticket.using_encoded(blake2_256)
// 	}
// }
