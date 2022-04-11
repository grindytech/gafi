#![cfg_attr(not(feature = "std"), no_std)]
use crate::weights::WeightInfo;
use frame_support::{
	dispatch::{DispatchResult, Vec},
	pallet_prelude::*,
	traits::{
		tokens::{ExistenceRequirement, WithdrawReasons},
		Currency,
	},
};
use frame_system::pallet_prelude::*;
use gafi_primitives::{
	currency::{centi, NativeToken::GAKI},
	pool::{Ticket, TicketType},
};
use gafi_primitives::{
	option_pool::{OptionPlayer, PackService},
	pool::{GafiPool, Level, Service},
};
pub use pallet::*;
use pallet_timestamp::{self as timestamp};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use gafi_primitives::pool::Ticket;

	use super::*;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config + timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type WeightInfo: WeightInfo;
		#[pallet::constant]
		type MaxNewPlayer: Get<u32>;
		#[pallet::constant]
		type MaxIngamePlayer: Get<u32>;
	}

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
		AlreadyOnStakingPool,
		IntoBalanceFail,
		ServiceNotSupport,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Joined(T::AccountId),
		Leaved(T::AccountId),
		ChargePoolService,
	}

	/*
		1. Charge player in the IngamePlayers
			1.1 Kick player when they can't pay
		2. Move all players from NewPlayer to IngamePlayers
	*/
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_block_number: BlockNumberFor<T>) {
			let _now = <timestamp::Pallet<T>>::get();

			if _now - Self::mark_time() >= Self::time_service().try_into().ok().unwrap() {
				let _ = Self::charge_ingame();
				let _ = Self::move_newplayer_to_ingame();
				MarkTime::<T>::put(_now);
				Self::deposit_event(<Event<T>>::ChargePoolService);
			}
		}
	}

	//** Storage **//
	#[pallet::storage]
	#[pallet::getter(fn max_player)]
	pub type MaxPlayer<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::type_value]
	pub fn DefaultMarkTime<T: Config>() -> T::Moment {
		<timestamp::Pallet<T>>::get()
	}
	#[pallet::storage]
	#[pallet::getter(fn mark_time)]
	pub type MarkTime<T: Config> = StorageValue<_, T::Moment, ValueQuery, DefaultMarkTime<T>>;

	#[pallet::type_value]
	pub fn DefaultTimeService() -> u128 {
		// 1 hour
		3_600_000u128
	}
	#[pallet::storage]
	#[pallet::getter(fn time_service)]
	pub type TimeService<T: Config> = StorageValue<_, u128, ValueQuery, DefaultTimeService>;

	#[pallet::storage]
	#[pallet::getter(fn player_count)]
	pub(super) type PlayerCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	pub(super) type NewPlayers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxNewPlayer>, ValueQuery>;

	#[pallet::storage]
	pub type IngamePlayers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxIngamePlayer>, ValueQuery>;

	#[pallet::type_value]
	pub(super) fn DefaultService<T: Config>() -> Service {
		Service { tx_limit: 4, discount: 60, value: 1000_000_000u128 }
	}
	#[pallet::storage]
	#[pallet::getter(fn services)]
	pub(super) type Services<T: Config> =
		StorageMap<_, Twox64Concat, PackService, Service, ValueQuery, DefaultService<T>>;

	#[pallet::storage]
	pub type Tickets<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Ticket<T::AccountId>>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub max_player: u32,
		pub services: [(PackService, u32, u8, u128); 3],
		pub time_service: u128,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			let base_fee: u128 = 75 * centi(GAKI); // 0.75 GAKI
			Self {
				max_player: 1000,
				services: [
					(PackService::Basic, 4, 60, (base_fee)),
					(PackService::Medium, 8, 70, (base_fee * 2)),
					(PackService::Max, u32::MAX, 80, (base_fee * 3)),
				],

				time_service: 3_600_000u128,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<MaxPlayer<T>>::put(self.max_player);

			for service in self.services.iter() {
				let new_service =
					Service { tx_limit: service.1, discount: service.2, value: service.3 };
				<Services<T>>::insert(service.0, new_service);
			}

			<MarkTime<T>>::put(<timestamp::Pallet<T>>::get());
			<TimeService<T>>::put(self.time_service);
		}
	}

	impl<T: Config> GafiPool<T::AccountId> for Pallet<T> {
		fn join(sender: T::AccountId, level: Level) -> DispatchResult {
			// make sure not exceed max players
			let new_player_count =
				Self::player_count().checked_add(1).ok_or(<Error<T>>::PlayerCountOverflow)?;

			ensure!(new_player_count <= Self::max_player(), <Error<T>>::ExceedMaxPlayer);
			{
				<NewPlayers<T>>::try_mutate(|newplayers| newplayers.try_push(sender.clone()))
					.map_err(|_| <Error<T>>::ExceedMaxNewPlayer)?;

				let service = Self::get_service(level);
				let double_fee = Self::u128_try_to_balance(service.value * 2)?;
				Self::change_fee(&sender, double_fee)?;
			}
			Self::join_pool(sender.clone(), level, new_player_count);
			Ok(())
		}

		fn leave(sender: T::AccountId) -> DispatchResult {
			let ticket = Tickets::<T>::get(sender.clone());
			ensure!(ticket != None, <Error<T>>::PlayerNotFound);
			let ticket = ticket.unwrap();
			if let Some(level) = Self::into_level(sender.clone()) {
				let join_time = ticket.join_time;
				let _now = Self::moment_to_u128(<timestamp::Pallet<T>>::get());
				let refund_fee = Self::calculate_ingame_refund_amount(_now, join_time, level)?;

				<NewPlayers<T>>::try_mutate(|players| {
					if let Some(ind) = players.iter().position(|id| id == &sender) {
						players.swap_remove(ind);
					}
					Ok(())
				})
				.map_err(|_: Error<T>| <Error<T>>::PlayerNotFound)?;
				<IngamePlayers<T>>::try_mutate(|players| {
					if let Some(ind) = players.iter().position(|id| id == &sender) {
						players.swap_remove(ind);
					}
					Ok(())
				})
				.map_err(|_: Error<T>| <Error<T>>::PlayerNotFound)?;

				let new_player_count =
					Self::player_count().checked_sub(1).ok_or(<Error<T>>::PlayerCountOverflow)?;
				Self::leave_pool(&sender, refund_fee, new_player_count);
			}
			Ok(())
		}

		fn get_service(level: Level) -> Service {
			match level {
				Level::Basic => todo!(),
				Level::Medium => todo!(),
				Level::Max => todo!(),
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/*
			* Set new MaxPlayer for the pool, MaxPlayer must be <= MaxNewPlayer and <= MaxIngamePlayer
			*/
		#[pallet::weight(0)]
		pub fn set_max_player(origin: OriginFor<T>, max_player: u32) -> DispatchResult {
			ensure_root(origin)?;
			// make sure new max_player not exceed the capacity of NewPlayers and IngamePlayers
			ensure!(max_player <= T::MaxNewPlayer::get(), <Error<T>>::ExceedMaxNewPlayer);
			ensure!(max_player <= T::MaxIngamePlayer::get(), <Error<T>>::ExceedMaxIngamePlayer);
			<MaxPlayer<T>>::put(max_player);
			Ok(())
		}

		/*
			* Set new pack service
			*/
		#[pallet::weight(0)]
		pub fn set_pack_service(
			origin: OriginFor<T>,
			pack: PackService,
			tx_limit: u32,
			discount: u8,
			service: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			let value = Self::balance_to_u128(service);
			let pack_service = Service { tx_limit, discount, value: value.unwrap() };
			Services::<T>::insert(pack, pack_service);
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn join_pool(sender: T::AccountId, level: Level, new_player_count: u32) {
		let _now = <timestamp::Pallet<T>>::get();
		let ticket = Ticket::<T::AccountId> {
			address: sender.clone(),
			join_time: Self::moment_to_u128(_now),
			ticket_type: TicketType::Upfront(level),
		};
		Tickets::<T>::insert(sender, ticket);
		<PlayerCount<T>>::put(new_player_count);
	}

	/*
		1. Calculate fee to refund
		2. Remove sender from Players and NewPlayers/IngamePlayers
	*/
	fn leave_pool(sender: &T::AccountId, refund_fee: BalanceOf<T>, new_player_count: u32) {
		// <Players<T>>::remove(sender);
		let _ = T::Currency::deposit_into_existing(sender, refund_fee);
		<PlayerCount<T>>::put(new_player_count);
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

	pub fn calculate_ingame_refund_amount(
		_now: u128,
		join_time: u128,
		level: Level,
	) -> Result<BalanceOf<T>, Error<T>> {
		let service = Self::get_service(level);

		let period_time = _now.saturating_sub(join_time);
		let mut fee: u128 = 0;
		if period_time < Self::time_service() {
			fee = service.value;
		} else {
			let extra = period_time % Self::time_service();
			let serive_fee = service.value;
			let actual_fee = serive_fee
				.saturating_mul(Self::time_service().saturating_sub(extra))
				.saturating_div(Self::time_service());
			fee = actual_fee;
		}

		match Self::u128_to_balance(fee) {
			Some(value) => Ok(value),
			None => Err(<Error<T>>::CanNotCalculateRefundFee),
		}
	}

	/*
	 */
	fn kick_ingame_player(player: &T::AccountId) -> Result<(), Error<T>> {
		// <Players<T>>::remove(player);
		<IngamePlayers<T>>::try_mutate(|players| {
			if let Some(ind) = players.iter().position(|id| id == player) {
				players.swap_remove(ind);
			}
			Ok(())
		})
		.map_err(|_: Error<T>| <Error<T>>::PlayerNotFound)?;

		let new_player_count =
			Self::player_count().checked_sub(1).ok_or(<Error<T>>::PlayerCountOverflow)?;
		<PlayerCount<T>>::put(new_player_count);
		Ok(())
	}

	fn charge_ingame() -> Result<(), Error<T>> {
		let ingame_players: Vec<T::AccountId> = IngamePlayers::<T>::get().into_inner();
		for player in ingame_players {
			if let Some(player_service) = Self::get_player_service(player.clone()) {
				match Self::change_fee(&player, player_service) {
					Ok(_) => {},
					Err(_) => {
						let _ = Self::kick_ingame_player(&player);
					},
				}
			}
		}
		Ok(())
	}

	fn move_newplayer_to_ingame() -> Result<(), Error<T>> {
		let new_players: Vec<T::AccountId> = NewPlayers::<T>::get().into_inner();
		for new_player in new_players {
			<IngamePlayers<T>>::try_append(new_player)
				.map_err(|_| <Error<T>>::ExceedMaxNewPlayer)?;
		}
		<NewPlayers<T>>::kill();
		Ok(())
	}

	fn into_level(player: T::AccountId) -> Option<Level> {
		if let Some(ticket) = Tickets::<T>::get(player) {
			if let TicketType::Upfront(level) = ticket.ticket_type {
				return Some(level);
			}
		}
		None
	}

	fn get_player_service(player: T::AccountId) -> Option<BalanceOf<T>> {
		if let Some(level) = Self::into_level(player) {
			let service = Self::get_service(level);
			return Self::u128_to_balance(service.value);
		}
		None
	}


	pub fn block_to_u64(input: T::BlockNumber) -> Option<u64> {
		TryInto::<u64>::try_into(input).ok()
	}

	pub fn balance_to_u64(input: BalanceOf<T>) -> Option<u64> {
		TryInto::<u64>::try_into(input).ok()
	}

	pub fn balance_to_u128(input: BalanceOf<T>) -> Option<u128> {
		TryInto::<u128>::try_into(input).ok()
	}

	pub fn u64_to_balance(input: u64) -> Option<BalanceOf<T>> {
		input.try_into().ok()
	}

	pub fn u128_to_balance(input: u128) -> Option<BalanceOf<T>> {
		input.try_into().ok()
	}

	pub fn u128_try_to_balance(input: u128) -> Result<BalanceOf<T>, Error<T>> {
		match input.try_into().ok() {
			Some(val) => Ok(val),
			None => Err(<Error<T>>::IntoBalanceFail)
		}
	}

	pub fn u64_to_block(input: u64) -> Option<T::BlockNumber> {
		input.try_into().ok()
	}

	pub fn moment_to_u128(input: T::Moment) -> u128 {
		sp_runtime::SaturatedConversion::saturated_into(input)
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

