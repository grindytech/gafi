#![cfg_attr(not(feature = "std"), no_std)]

use sp_api::Encode;

sp_api::decl_runtime_apis! {
	pub trait GameNftsRpcRuntimeApi<AccountId: Encode> {
		fn create_collection() -> u128;
	}
}
