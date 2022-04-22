use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::{currency::{unit, NativeToken::GAKI}, constant::ID};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Ticket<AccountId> {
	pub address: AccountId,
	pub join_time: u128,
	pub ticket_type: TicketType,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TicketType {
	Upfront(Level),
	Staking(Level),
	Sponsored(ID),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Level {
	Basic,
	Medium,
	Advance,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Service {
	pub tx_limit: u32, // max number of discounted transaction user can use in TimeService 
	pub discount: u8,  // percentage of discount
	pub value: u128,
}

pub trait FlexPool<AccountId> {
	fn join(sender: AccountId, level: Level) -> DispatchResult;
	fn leave(sender: AccountId) -> DispatchResult;
	fn get_service(level: Level) -> Option<Service>;
}

pub trait StaticPool<AccountId> {
	fn join(sender: AccountId, pool_id: ID) -> DispatchResult;
	fn leave(sender: AccountId) -> DispatchResult;
	fn get_service(pool_id: ID) -> Option<Service>;
}

impl Service {
	pub fn new(ticket: TicketType) -> Self {
		match ticket {
			TicketType::Upfront(level) => match level {
				Level::Basic => Service { tx_limit: 100, discount: 30, value: 5 * unit(GAKI) },
				Level::Medium => Service { tx_limit: 100, discount: 50, value: 7 * unit(GAKI) },
				Level::Advance => Service { tx_limit: 100, discount: 70, value: 10 * unit(GAKI) },
			},
			TicketType::Staking(level) => match level {
				Level::Basic => Service { tx_limit: 100, discount: 30, value: 1000 * unit(GAKI) },
				Level::Medium => Service { tx_limit: 100, discount: 50, value: 1500 * unit(GAKI) },
				Level::Advance => Service { tx_limit: 100, discount: 70, value: 2000 * unit(GAKI) },
			},
			TicketType::Sponsored(_) => {
				return Service { tx_limit: 100, discount: 30, value: unit(GAKI) }
			},
		}
	}
}

pub trait PlayerTicket<AccountId> {
	fn use_ticket(player: AccountId) -> Option<TicketType>;
	fn get_service(ticket: TicketType) -> Option<Service>;
}

pub trait MasterPool<AccountId> {
	fn remove_player(player: &AccountId);
	fn get_timeservice() -> u128;
	fn get_marktime() -> u128;
}
