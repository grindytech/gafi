use crate::{constant::ID, pool::Service, ticket::Ticket};
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_runtime::{Permill, RuntimeDebug};
use sp_std::{prelude::*, vec, vec::Vec};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	// SBP REVIEW - Default implementation 
	Default, Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct SystemService {
	pub id: ID,
	pub service: Service,
	pub value: u128,
}

impl SystemService {
	pub fn new(id: ID, tx_limit: u32, discount: Permill, value: u128) -> Self {
		SystemService {
			id,
			service: Service { tx_limit, discount },
			value,
		}
	}
	// SBP REVIEW -
	// We see that SystemService already has a derivation for Default, and then another method with the same name
	// as Default::default(). This may lead to code ambiguity. 

	// Recommend to move this code to a manual implementation of default: 
	// ```rust 
	// impl Default for SystemService  { .. } 
	pub fn default() -> Self {
		Self {
			id: [0; 32],
			service: Service {
				tx_limit: 0,
				discount: Permill::from_percent(0),
			},
			value: 0_u128,
		}
	}
}

pub trait SystemPool<AccountIdLookup, AccountId> {
	fn join(sender: AccountIdLookup, pool_id: ID) -> DispatchResult;
	fn leave(sender: AccountIdLookup) -> DispatchResult;
	fn get_service(pool_id: ID) -> Option<SystemService>;
	fn get_ticket(sender: &AccountId) -> Option<Ticket<AccountId>>;
}

impl<AccountIdLookup, AccountId> SystemPool<AccountIdLookup, AccountId> for () {
	fn join(_sender: AccountIdLookup, _pool_id: ID) -> DispatchResult {
		Ok(())
	}
	fn leave(_sender: AccountIdLookup) -> DispatchResult {
		Ok(())
	}
	fn get_service(_pool_id: ID) -> Option<SystemService> {
		Default::default()
	}
	fn get_ticket(_sender: &AccountId) -> Option<Ticket<AccountId>> {
		Default::default()
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
pub struct SystemServicePack {
	pub data: Vec<(ID, SystemService)>,
}

impl SystemServicePack {
	pub fn new(data: Vec<(ID, SystemService)>) -> Self {
		Self { data }
	}

	pub fn default() -> Self {
		Self { data: vec![] }
	}
}

pub trait SystemDefaultServices {
	fn get_default_services() -> SystemServicePack;
}
