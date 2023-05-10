use crate::*;
use pallet_nfts::CollectionRole;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Returns true if a specified account has a provided role within that game.
	///
	/// - `game_id`: A game to check the role in.
	/// - `account_id`: An account to check the role for.
	/// - `role`: A role to validate.
	///
	/// Returns boolean.
	pub(crate) fn has_role(
		game_id: &T::GameId,
		account_id: &T::AccountId,
		role: CollectionRole,
	) -> bool {
		GameRoleOf::<T, I>::get(&game_id, &account_id)
			.map_or(false, |roles| roles.has_role(role))
	}
}
