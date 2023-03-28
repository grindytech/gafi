use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use pallet_nfts::ItemMetadata;
use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter};
use sp_core::{Get, U256};
use sp_runtime::AccountId32;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(StringLimit))]
#[codec(mel_bound(DepositBalance: MaxEncodedLen))]
pub struct UpgradeData<DepositBalance, StringLimit: Get<u32>> {
	token_id: U256,
	data: ItemMetadata<DepositBalance, StringLimit>,
}

pub trait GameNFT<DepositBalance, StringLimit: Get<u32>, AccountId> {

	fn set_upgrade() -> Result<(), ()>;

	fn upgrade(
		token_id: U256,
		address: AccountId,
		upgrade_data: UpgradeData<DepositBalance, StringLimit>,
	) -> Result<(), ()>;

	fn approve_upgrade(token_id: U256, address: AccountId) -> Result<(), ()>;

	fn allow_combine(collection_id: u32) -> Result<(), ()>;

	fn combine(token_id: U256, address: AccountId) -> Result<(), ()>;
}
