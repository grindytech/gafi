use sp_core::{U256};
use sp_runtime::AccountId32;

pub trait NFT {

}

pub trait GameNFT {
	fn upgrade(token_id: U256, address: AccountId32) -> Result<(), ()>;
	
}
