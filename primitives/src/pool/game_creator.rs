use sp_core::H160;

pub trait GetGameCreator<AccountId> {
	fn get_game_creator(contract: &H160) -> Option<AccountId>;
}

impl<AccountId> GetGameCreator<AccountId> for () {
	fn get_game_creator(_contract: &H160) -> Option<AccountId> {
		None
	}
}
