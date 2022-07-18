pub trait PlayersTime<AccountId> {
	fn add_time_joined_upfront(player: AccountId, time: u128);
}
