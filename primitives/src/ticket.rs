use crate::{
    constant::ID,
    pool::{Service},
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
	System(SystemTicket),
    Custom(CustomTicket),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum SystemTicket {
	Upfront(TicketLevel),
	Staking(TicketLevel),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CustomTicket {
    Sponsored(ID),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Copy, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TicketLevel {
	Basic,
	Medium,
	Advance,
}

/// Holding the number of tickets to restrict player transaction
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct TicketInfo {
    pub ticket_type: TicketType,
    pub tickets: u32,
	pub pool_id: Option<ID>
}

impl TicketInfo {
    /// reduce tickets by 1
    pub fn withdraw_ticket(&self) -> Option<Self> {
        if let Some(new_tickets) = self.tickets.checked_sub(1) {
            return Some(TicketInfo {
                tickets: new_tickets,
                ticket_type: self.ticket_type,
            });
        }
        None
    }

    /// renew ticket
    pub fn renew_ticket(&self, new_remain: u32) -> Self {
        TicketInfo {
            tickets: new_remain,
            ticket_type: self.ticket_type,
        }
    }
}

pub trait PlayerTicket<AccountId> {
    fn use_ticket(player: AccountId, pool_id_key: Option<ID>) -> Option<TicketType>;
    fn get_service(ticket: TicketType) -> Option<Service>;
    fn get_targets(pool_id: ID) -> Vec<H160>;
}

impl<AccountId> PlayerTicket<AccountId> for () {
    fn use_ticket(_player: AccountId, _pool_id_key: Option<ID>) -> Option<TicketType> {
        None
    }

    fn get_service(_ticket: TicketType) -> Option<Service> {
        None
    }

    fn get_targets(_pool_id: ID) -> Vec<H160> {
        [].to_vec()
    }
}
