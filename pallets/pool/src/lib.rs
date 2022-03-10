#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::{DispatchResult, Vec},
	pallet_prelude::*,
	traits::{
		tokens::{ExistenceRequirement, WithdrawReasons},
		Currency,
	},
};

use sp_runtime::RuntimeDebug;

use frame_system::pallet_prelude::*;



#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum PackService {
	Basic,
	Medium,
	Max,
}

pub trait PackServiceProvider<T: Config> {
	fn get_service(service: PackService) -> Service<T>;
}


#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait AuroraZone<T: Config> {
	fn is_in_aurora_zone(player: &T::AccountId) -> Option<Player<T>>;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Service<T: Config> {
		tx_limit: u8, // max number of transaction per minute
		discount: u8,
		service: BalanceOf<T>,
	}

	impl <T: Config> MaxEncodedLen for Service<T> {
		fn max_encoded_len() -> usize {
			1000
		}
	}

	impl MaxEncodedLen for PackService {
		fn max_encoded_len() -> usize {
			1000
		}
	}

	// Struct, Enum
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Player<T: Config> {
		address: T::AccountId,
		join_block: u64,
		service: PackService,
	}

	impl<T: Config> MaxEncodedLen for Player<T> {
		fn max_encoded_len() -> usize {
			1000
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type MaxNewPlayer: Get<u32>;

		#[pallet::constant]
		type MaxIngamePlayer: Get<u32>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		PlayerNotFound,
		PlayerAlreadyJoin,

		PlayerCountOverflow,
		ExceedMaxPlayer,
		ExceedMaxNewPlayer,
		CanNotClearNewPlayers,
		ExceedMaxIngamePlayer,
		CanNotCalculateRefundFee,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PlayerJoinPool(T::AccountId),
		PlayerLeavePool(T::AccountId),
	}

	/*
		1. Charge player in the IngamePlayers
			1.1 Kick player when they can't pay
		2. Move all players from NewPlayer to IngamePlayers
	*/
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(block_number: BlockNumberFor<T>) {
			let mark_block = Self::mark_block();

			if let Some(block) = Self::block_to_u64(block_number) {
				if block % mark_block == 0 {
					let _ = Self::charge_ingame();
					let _ = Self::move_newplayer_to_ingame();
				}
			}
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn max_player)]
	pub type MaxPlayer<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn mark_block)]
	pub type MarkBlock<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_fee)]
	pub type PoolFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	// Store all players join the pool
	#[pallet::storage]
	#[pallet::getter(fn players)]
	pub(super) type Players<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Player<T>>;

	#[pallet::storage]
	#[pallet::getter(fn player_count)]
	pub(super) type PlayerCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn new_players)]
	pub(super) type NewPlayers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxNewPlayer>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn ingame_players)]
	pub type IngamePlayers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxIngamePlayer>, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn services)]
	pub(super) type Services<T: Config> = StorageMap<_, Twox64Concat, PackService, Service<T>>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub max_player: u32,
		pub mark_block: u64,
		pub pool_fee: BalanceOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { max_player: 1000, mark_block: 30, pool_fee: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<MaxPlayer<T>>::put(self.max_player);
			<MarkBlock<T>>::put(self.mark_block);
			<PoolFee<T>>::put(self.pool_fee);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn join(origin: OriginFor<T>, service: PackService) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::join_pool(sender.clone(), service)?;
			let pool_fee = Self::pool_fee();
			let double_fee = pool_fee * 2u32.into();
			Self::change_fee(&sender, double_fee)?;
			Self::deposit_event(Event::PlayerJoinPool(sender));
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn leave(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::leave_pool(&sender)?;
			Self::deposit_event(Event::PlayerLeavePool(sender));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn join_pool(sender: T::AccountId, service: PackService) -> Result<(), Error<T>> {
			// make sure player not re-join
			ensure!(Self::players(sender.clone()) == None, <Error<T>>::PlayerAlreadyJoin);
			// make sure not exceed max players
			let new_player_count =
				Self::player_count().checked_add(1).ok_or(<Error<T>>::PlayerCountOverflow)?;
			ensure!(new_player_count <= Self::max_player(), <Error<T>>::ExceedMaxPlayer);
			// make sure not exceed max new players
			<NewPlayers<T>>::try_mutate(|newplayers| newplayers.try_push(sender.clone()))
				.map_err(|_| <Error<T>>::ExceedMaxNewPlayer)?;
			let block_number = Self::get_block_number();
			let player = Player::<T> { address: sender.clone(), join_block: block_number, service };
			<Players<T>>::insert(sender, player);
			<PlayerCount<T>>::put(new_player_count);
			Ok(())
		}

		pub fn change_fee(sender: &T::AccountId, fee: BalanceOf<T>) -> DispatchResult {
			let withdraw = T::Currency::withdraw(
				&sender,
				fee,
				WithdrawReasons::RESERVE,
				ExistenceRequirement::KeepAlive,
			);

			match withdraw {
				Ok(_) => Ok(()),
				Err(err) => Err(err),
			}
		}

		/*
			1. Calculate fee to refund
			2. Remove sender from Players and NewPlayers/IngamePlayers
		*/
		fn leave_pool(sender: &T::AccountId) -> Result<(), Error<T>> {
			if let Some(player) = Self::players(sender) {
				let join_block = player.join_block;
				let block_number = Self::get_block_number();
				let refund_fee: BalanceOf<T>;
				let range_block = block_number - join_block;

				if range_block < Self::mark_block() {
					<NewPlayers<T>>::try_mutate(|players| {
						if let Some(ind) = players.iter().position(|id| id == sender) {
							players.swap_remove(ind);
						}
						Ok(())
					})
					.map_err(|_: Error<T>| <Error<T>>::PlayerNotFound)?;
					refund_fee = Self::pool_fee();
				} else {
					<IngamePlayers<T>>::try_mutate(|players| {
						if let Some(ind) = players.iter().position(|id| id == sender) {
							players.swap_remove(ind);
						}
						Ok(())
					})
					.map_err(|_: Error<T>| <Error<T>>::PlayerNotFound)?;
					refund_fee = Self::calculate_ingame_refund_amount(join_block, block_number)?;
				}
				<Players<T>>::remove(sender);
				let _ = T::Currency::deposit_into_existing(sender, refund_fee);
			} else {
				return Err(<Error<T>>::PlayerNotFound);
			}
			Ok(())
		}

		pub fn calculate_ingame_refund_amount(
			join_block: u64,
			block_number: u64,
		) -> Result<BalanceOf<T>, Error<T>> {
			let range_block = block_number - join_block;
			let extra = range_block % Self::mark_block();
			if let Some(fee) = Self::balance_to_u64(Self::pool_fee()) {
				let actual_fee = fee * (Self::mark_block() - extra) / Self::mark_block();
				if let Some(result) = Self::u64_to_balance(actual_fee) {
					return Ok(result);
				}
			}
			return Err(<Error<T>>::CanNotCalculateRefundFee);
		}

		/*
		 */
		fn kick_ingame_player(player: &T::AccountId) -> Result<(), Error<T>> {
			<Players<T>>::remove(player);
			<IngamePlayers<T>>::try_mutate(|players| {
				if let Some(ind) = players.iter().position(|id| id == player) {
					players.swap_remove(ind);
				}
				Ok(())
			})
			.map_err(|_: Error<T>| <Error<T>>::PlayerNotFound)?;
			Ok(())
		}

		fn charge_ingame() -> Result<(), Error<T>> {
			let ingame_players: Vec<T::AccountId> = Self::ingame_players().into_inner();
			for player in ingame_players {
				match Self::change_fee(&player, Self::pool_fee()) {
					Ok(_) => {},
					Err(_) => {
						let _ = Self::kick_ingame_player(&player);
					},
				}
			}
			Ok(())
		}

		fn move_newplayer_to_ingame() -> Result<(), Error<T>> {
			let new_players: Vec<T::AccountId> = Self::new_players().into_inner();
			for new_player in new_players {
				<IngamePlayers<T>>::try_append(new_player)
					.map_err(|_| <Error<T>>::ExceedMaxNewPlayer)?;
			}
			<NewPlayers<T>>::kill();
			Ok(())
		}

		pub fn block_to_u64(input: T::BlockNumber) -> Option<u64> {
			TryInto::<u64>::try_into(input).ok()
		}

		pub fn balance_to_u64(input: BalanceOf<T>) -> Option<u64> {
			TryInto::<u64>::try_into(input).ok()
		}

		pub fn u64_to_balance(input: u64) -> Option<BalanceOf<T>> {
			input.try_into().ok()
		}

		pub fn u64_to_block(input: u64) -> Option<T::BlockNumber> {
			input.try_into().ok()
		}

		/*
			Return current block number otherwise return 0
		*/
		pub fn get_block_number() -> u64 {
			let block_number = <frame_system::Pallet<T>>::block_number();
			if let Some(block) = Self::block_to_u64(block_number) {
				return block;
			}
			return 0u64;
		}
	}

	impl<T: Config> AuroraZone<T> for Pallet<T> {
		fn is_in_aurora_zone(player: &T::AccountId) -> Option<Player<T>> {
			Self::players(player)
		}
	}

}

#[cfg(feature = "std")]
impl<T: Config> GenesisConfig<T> {
	pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
		<Self as GenesisBuild<T>>::build_storage(self)
	}

	pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
		<Self as GenesisBuild<T>>::assimilate_storage(self, storage)
	}
}
