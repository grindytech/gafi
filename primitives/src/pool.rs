
pub trait OptionPool<AccountId> {
    fn is_optioning(player: AccountId) -> bool;
}

pub trait SkingPool<AccountId> {
    fn is_staking(player: AccountId) -> bool;
}
