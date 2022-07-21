pub trait PlayersTime<AccountId> {
	fn add_time_joined_upfront(player: AccountId, time: u128);
}

pub trait PlayerJoinedPoolStatistic<AccountId> {
	fn get_total_time_joined_upfront(player: AccountId) -> u128;
}
