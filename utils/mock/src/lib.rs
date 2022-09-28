pub mod pool;
use gafi_primitives::currency::{unit, NativeToken::GAKI};

pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
pub const INIT_TIMESTAMP: u64 = 30_000;
pub const AN_HOUR: u128 = 60 * 60_000u128; // 1 hour

pub fn one_mil_gaki() -> u128 {
	1_000_000 * unit(GAKI)
}
