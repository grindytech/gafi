use crate::constant::ID;

pub trait WhitelistPool<AccountId> {
	fn join_pool(sender: &AccountId, pool_id: ID) -> Result<(), &'static str>;
}

impl<AccountId> WhitelistPool<AccountId> for () {
    fn join_pool(_sender: &AccountId, _pool_id: ID) -> Result<(), &'static str> {
        Err("default")
    }
}

pub trait IWhitelist<AccountId> {
    fn is_whitelist(pool_id: ID) -> bool;
    fn insert_whitelist(pool_id: ID, player: AccountId) -> Result<(), &'static str>;
}
