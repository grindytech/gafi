use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use crate::pool::TicketType;

/// Holding the number of tickets to restrict player transaction
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct TicketInfo {
    pub ticket_type: TicketType,
    pub tickets: u32,
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
    pub  fn renew_ticket(&self, new_remain: u32) -> Self {
        TicketInfo {
            tickets: new_remain,
            ticket_type: self.ticket_type,
        }
    }
}

