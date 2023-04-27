use frame_support::dispatch::DispatchResult;
use sp_std::vec::Vec;

use crate::common::ID;

pub trait Name<AccountId> {
	fn set_name(origin: AccountId, asset_id: ID, name: Vec<u8>) -> DispatchResult;
	fn clear_name(origin: AccountId, asset_id: ID) -> DispatchResult;
	fn kill_name(origin: AccountId, asset_id: ID) -> DispatchResult;
}

impl<AccountId> Name<AccountId> for () {
	fn set_name(_origin: AccountId, _asset_id: ID, _name: Vec<u8>) -> DispatchResult {
		Ok(())
	}

	fn clear_name(_origin: AccountId, _asset_idd: ID) -> DispatchResult {
		Ok(())
	}

	fn kill_name(_origin: AccountId, _asset_idd: ID) -> DispatchResult {
		Ok(())
	}
}
