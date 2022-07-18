#![cfg_attr(not(feature = "std"), no_std)]

use sp_api::Encode;

sp_api::decl_runtime_apis! {
	pub trait PlayerRuntimeRPCApi<AccountId: Encode> {
		fn get_total_time_joined_upfront(player: AccountId) -> u128;
	}
}
