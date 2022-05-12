use sp_core::H160;

pub trait GetGameCreator<AccountId> {
    fn get_game_creator(contract: &H160) -> Option<AccountId>;
}
