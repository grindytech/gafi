use crate::{
	constant::ID,
	currency::{unit, NativeToken::GAKI},
};
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

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
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct FlexService {
	pub service: Service,
	pub value: u128,
}

impl FlexService {
	pub fn new(tx_limit: u32, discount: u8, value: u128) -> Self {
		FlexService {
			service: Service { tx_limit, discount },
			value,
		}
	}
}

pub trait FlexPool<AccountId> {
	fn join(sender: AccountId, level: Level) -> DispatchResult;
	fn leave(sender: AccountId) -> DispatchResult;
	fn get_service(level: Level) -> Option<FlexService>;
}

// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct StaticService<AccountId> {
	pub service: Service,
	pub sponsor: AccountId,
	pub targets: Vec<H160>,
}

impl<AccountId> StaticService<AccountId> {
	pub fn new(targets: Vec<H160>, tx_limit: u32, discount: u8, sponsor: AccountId) -> Self {
		StaticService {
			targets,
			service: Service { tx_limit, discount },
			sponsor,
		}
	}
}

pub trait StaticPool<AccountId> {
	fn join(sender: AccountId, pool_id: ID) -> DispatchResult;
	fn leave(sender: AccountId) -> DispatchResult;
	fn get_service(pool_id: ID) -> Option<StaticService<AccountId>>;
}

pub trait PlayerTicket<AccountId> {
	fn use_ticket(player: AccountId) -> Option<TicketType>;
	fn get_service(ticket: TicketType) -> Option<Service>;
	fn get_targets(pool_id: ID) -> Vec<H160>;
}

pub trait MasterPool<AccountId> {
	fn remove_player(player: &AccountId);
	fn get_timeservice() -> u128;
	fn get_marktime() -> u128;
}

impl<AccountId> MasterPool<AccountId> for () {
	fn remove_player(_player: &AccountId) {}
	fn get_timeservice() -> u128 {
		30 * 60_000u128 // 30 minutes
	}
	fn get_marktime() -> u128 {
		u128::default()
	}
}
