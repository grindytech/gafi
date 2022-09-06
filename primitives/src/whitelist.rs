use crate::constant::ID;

pub trait WhitelistPool<AccountId> {
	fn join_pool(sender: &AccountId, pool_id: ID) -> Result<(), &'static str>;
}