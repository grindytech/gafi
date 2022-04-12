use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::currency::{unit, NativeToken::GAKI};

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
	Sponsored(Level),
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
	pub tx_limit: u32, // max number of transaction per minute
	pub discount: u8,  // percentage of discount
	pub value: u128,
}

impl Service {
	pub fn new(ticket: TicketType) -> Self {
		match ticket {
			TicketType::Upfront(level) => match level {
				Level::Basic => Service { tx_limit: 4, discount: 30, value: unit(GAKI) },
				Level::Medium => Service { tx_limit: 8, discount: 50, value: 2 * unit(GAKI) },
				Level::Advance => Service { tx_limit: u32::MAX, discount: 70, value: 3 * unit(GAKI) },
			},
			TicketType::Staking(level) => match level {
				Level::Basic => Service { tx_limit: 4, discount: 30, value: 1000 * unit(GAKI) },
				Level::Medium => Service { tx_limit: 8, discount: 50, value: 2 * 1000 * unit(GAKI) },
				Level::Advance => Service { tx_limit: u32::MAX, discount: 70, value: 3 * 1000 * unit(GAKI) },
			},
			TicketType::Sponsored(level) => match level {
				Level::Basic => Service { tx_limit: 4, discount: 30, value: unit(GAKI) },
				Level::Medium => Service { tx_limit: 8, discount: 50, value: 2 * unit(GAKI) },
				Level::Advance => Service { tx_limit: u32::MAX, discount: 70, value: 3 * unit(GAKI) },
			},
		}
	}
}

pub trait GafiPool<AccountId> {
	fn join(sender: AccountId, level: Level) -> DispatchResult;
	fn leave(sender: AccountId) -> DispatchResult;
	fn get_service(level: Level) -> Service;
}

pub trait PlayerTicket<AccountId> {
	fn get_player_ticket(player: AccountId) -> Option<TicketType>;
	fn get_ticket(ticket: TicketType) -> Service;
}

pub trait MasterPool<AccountId> {
	fn remove_player(player: &AccountId);
}
