use crate::constant::ID;

pub trait WhitelistPool<AccountId> {
	fn join_pool(sender: &AccountId, pool_id: ID) -> Result<(), &'static str>;
    fn is_joined_pool(sender: &AccountId, pool_id: ID) -> bool;
}

impl<AccountId> WhitelistPool<AccountId> for () {
    fn join_pool(_sender: &AccountId, _pool_id: ID) -> Result<(), &'static str> {
        Err("default")
    }

    fn is_joined_pool(_sender: &AccountId, _pool_id: ID) -> bool {
        false
    }
}

pub trait IWhitelist<AccountId> {
    fn is_whitelist(pool_id: ID) -> bool;
    fn insert_whitelist(pool_id: ID, player: AccountId) -> Result<(), &'static str>;
}

impl<AccountId> IWhitelist<AccountId> for () {
    fn is_whitelist(_pool_id: ID) -> bool {
        false
    }

    fn insert_whitelist(_pool_id: ID, _player: AccountId) -> Result<(), &'static str> {
        Err("default")
    }
}