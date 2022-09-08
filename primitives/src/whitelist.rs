use crate::constant::ID;

pub trait WhitelistPool<AccountId> {
	fn join_pool(sender: &AccountId, pool_id: ID) -> Result<(), &'static str>;
}

pub trait WhitelistSponsor<AccountId> {
    fn is_pool(pool_id: ID) -> bool;
    fn get_pool_owner(pool_id: ID) -> Result<AccountId, &'static str>;
}

impl<AccountId> WhitelistPool<AccountId> for () {
    fn join_pool(_sender: &AccountId, _pool_id: ID) -> Result<(), &'static str> {
        Err("default")
    }
}

impl<AccountId> WhitelistSponsor<AccountId> for () {
    fn is_pool(_pool_id: ID) -> bool {
        false
    }

    fn get_pool_owner(_pool_id: ID) -> Result<AccountId, &'static str> {
        Err("default")
    }
}
