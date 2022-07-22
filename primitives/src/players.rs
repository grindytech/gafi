pub trait PlayersTime<AccountId> {
	fn add_time_joined_upfront(player: AccountId, time: u128);
}

impl<AccountId> PlayersTime<AccountId> for () {
	fn add_time_joined_upfront(_player: AccountId, _time: u128) {}
}
