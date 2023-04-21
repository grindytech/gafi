pub trait PlayersTime<AccountId> {
	fn add_time_joined_upfront(player: AccountId, time: u128);
	fn add_time_joined_staking(player: AccountId, time: u128);
}

impl<AccountId> PlayersTime<AccountId> for () {
	fn add_time_joined_upfront(_player: AccountId, _time: u128) {}
	fn add_time_joined_staking(_player: AccountId, _time: u128) {}
}

pub trait PlayerJoinedPoolStatistic<AccountId> {
	fn get_total_time_joined_upfront(player: &AccountId) -> u128;
	fn get_total_time_joined_staking(player: &AccountId) -> u128;
}
