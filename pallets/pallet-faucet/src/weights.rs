
//! Autogenerated weights for `pallet_faucet`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-04-26, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/gafi-node
// benchmark
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_faucet
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --json-file=raw.json
// --output
// ./pallets/benchmarks/pallet-faucet/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn faucet(s: u32, ) -> Weight;
	fn donate(s: u32, ) -> Weight;
}
/// Weight functions for `pallet_faucet`.
pub struct FaucetWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for FaucetWeight<T> {
	// Storage: Faucet GenesisAccounts (r:1 w:0)
	// Storage: Faucet FaucetAmount (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn faucet(_b: u32, ) -> Weight {
		(39_500_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: System Account (r:2 w:2)
	// Storage: Faucet GenesisAccounts (r:1 w:0)
	// Storage: System Number (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn donate(_b: u32, ) -> Weight {
		(31_411_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
}

impl WeightInfo for () {
	fn faucet(_b: u32, ) -> Weight {
		(39_500_000 as Weight)
		.saturating_add(RocksDbWeight::get().reads(7 as Weight))
		.saturating_add(RocksDbWeight::get().writes(3 as Weight))
	}

	fn donate(_b: u32,) -> Weight {
		(31_411_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(7 as Weight))
			.saturating_add(RocksDbWeight::get().writes(4 as Weight))
	}
}
