use crate::*;
use pallet_nfts::CollectionRole;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Returns true if a specified account has a provided role within that game.
	///
	/// - `game`: A game to check the role in.
	/// - `who`: An account to check the role for.
	/// - `role`: A role to validate.
	///
	/// Returns boolean.
	pub(crate) fn has_role(
		game: &T::GameId,
		who: &T::AccountId,
		role: CollectionRole,
	) -> bool {
		GameRoleOf::<T, I>::get(&game, &who)
			.map_or(false, |roles| roles.has_role(role))
	}
}
