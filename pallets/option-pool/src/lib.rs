#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
pub mod pool;
use crate::pool::{AuroraZone, PackService, PackServiceProvider, Player, Service};
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
use pallet_timestamp::{self as timestamp};
use aurora_primitives::{centi, currency::NativeToken::AUX};
use sp_runtime::{Perbill};

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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PlayerJoinPool(T::AccountId),
		PlayerLeavePool(T::AccountId),
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

	// Store all players join the pool
	#[pallet::storage]
	pub(super) type Players<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Player<T::AccountId>>;

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
	pub(super) fn DefaultService<T: Config>() -> Service<BalanceOf<T>> {
		Service { tx_limit: 4, discount: 60, service: 1000_000_000u128.try_into().ok().unwrap() }
	}
	#[pallet::storage]
	#[pallet::getter(fn services)]
	pub(super) type Services<T: Config> = StorageMap<
		_,
		Twox64Concat,
		PackService,
		Service<BalanceOf<T>>,
		ValueQuery,
		DefaultService<T>,
	>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub max_player: u32,
		pub services: [(PackService, u8, u8, BalanceOf<T>); 3],

		pub time_service: u128,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			let base_fee: u128 = 75 * centi(AUX); // 0.75 AUX
			let convert_default_fee = |fee: u128| -> BalanceOf<T> { fee.try_into().ok().unwrap() };
			Self {
				max_player: 1000,
				services: [
					(PackService::Basic, 4, 60, convert_default_fee(base_fee)),
					(PackService::Medium, 8, 70, convert_default_fee(base_fee * 2)),
					(PackService::Max, u8::MAX, 80, convert_default_fee(base_fee * 3)),
				],

				time_service: 3_600_000u128,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<MaxPlayer<T>>::put(self.max_player);

			for service in self.services.iter() {
				let new_service =
					Service { tx_limit: service.1, discount: service.2, service: service.3 };
				<Services<T>>::insert(service.0, new_service);
			}

			<MarkTime<T>>::put(<timestamp::Pallet<T>>::get());
			<TimeService<T>>::put(self.time_service);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/*
			* player join the pool
			* 1. Add new player to NewPlayer
			* 2. charge double service fee when they join
			*/
		#[pallet::weight(<T as pallet::Config>::WeightInfo::join(100u32))]
		pub fn join(origin: OriginFor<T>, pack: PackService) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// make sure player not re-join
			ensure!(Players::<T>::get(sender.clone()) == None, <Error<T>>::PlayerAlreadyJoin);
			// make sure not exceed max players
			let new_player_count =
				Self::player_count().checked_add(1).ok_or(<Error<T>>::PlayerCountOverflow)?;
			ensure!(new_player_count <= Self::max_player(), <Error<T>>::ExceedMaxPlayer);
			{
				<NewPlayers<T>>::try_mutate(|newplayers| newplayers.try_push(sender.clone()))
					.map_err(|_| <Error<T>>::ExceedMaxNewPlayer)?;
				let pack_service = Services::<T>::get(pack);
				let double_fee = pack_service.service * 2u32.into();
				Self::change_fee(&sender, double_fee)?;
			}

			Self::join_pool(sender.clone(), pack, new_player_count);
			Self::deposit_event(Event::PlayerJoinPool(sender));

			Ok(())
		}

		/*
			* player leave the pool
			* 1. remove player from storages
			* 2. refund appropriate amount (the maximum amount they receive is 'service_fee'/2)
			*/
		#[pallet::weight(<T as pallet::Config>::WeightInfo::leave(100u32))]
		pub fn leave(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let player = Players::<T>::get(sender.clone());
			ensure!(player != None, <Error<T>>::PlayerNotFound);
			let player = player.unwrap();

			{
				let join_time = player.join_time;
				let _now = Self::moment_to_u128(<timestamp::Pallet<T>>::get());
				let refund_fee =
					Self::calculate_ingame_refund_amount(_now, join_time, player.service)?;

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

			Self::deposit_event(Event::PlayerLeavePool(sender));
			Ok(())
		}

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
			tx_limit: u8,
			discount: u8,
			service: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let pack_service = Service { tx_limit, discount, service };
			Services::<T>::insert(pack, pack_service);
			Ok(())
		}
	}

	impl<T: Config> AuroraZone<T::AccountId> for Pallet<T> {
		fn is_in_aurora_zone(player: &T::AccountId) -> Option<Player<T::AccountId>> {
			Players::<T>::get(player)
		}
	}

	impl<T: Config> PackServiceProvider<BalanceOf<T>> for Pallet<T> {
		fn get_service(service: PackService) -> Option<Service<BalanceOf<T>>> {
			Some(Services::<T>::get(service))
		}
	}
}

impl<T: Config> Pallet<T> {
	fn join_pool(sender: T::AccountId, pack: PackService, new_player_count: u32) {
		let _now = <timestamp::Pallet<T>>::get();
		let player = Player::<T::AccountId> {
			address: sender.clone(),
			join_time: Self::moment_to_u128(_now),
			service: pack,
		};
		<Players<T>>::insert(sender, player);
		<PlayerCount<T>>::put(new_player_count);
	}

	/*
		1. Calculate fee to refund
		2. Remove sender from Players and NewPlayers/IngamePlayers
	*/
	fn leave_pool(sender: &T::AccountId, refund_fee: BalanceOf<T>, new_player_count: u32) {
		<Players<T>>::remove(sender);
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
		service: PackService,
	) -> Result<BalanceOf<T>, Error<T>> {
		let period_time = _now.saturating_sub(join_time);
		if period_time < Self::time_service() {
			let pack = Services::<T>::get(service);
			return Ok(pack.service);
		}
		let extra = period_time % Self::time_service();

		let service = Services::<T>::get(service);
		if let Some(fee) = Self::balance_to_u128(service.service) {
			let actual_fee = fee.saturating_mul(Self::time_service().saturating_sub(extra)).saturating_div(Self::time_service());
			if let Some(result) = Self::u128_to_balance(actual_fee) {
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

		let new_player_count =
				Self::player_count().checked_sub(1).ok_or(<Error<T>>::PlayerCountOverflow)?;
		<PlayerCount<T>>::put(new_player_count);
		Ok(())
	}

	fn charge_ingame() -> Result<(), Error<T>> {
		let ingame_players: Vec<T::AccountId> = IngamePlayers::<T>::get().into_inner();
		for player in ingame_players {
			if let Some(ingame_player) = Players::<T>::get(&player) {
				let pack = Services::<T>::get(ingame_player.service);
				match Self::change_fee(&player, pack.service) {
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

#[cfg(feature = "std")]
impl<T: Config> GenesisConfig<T> {
	pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
		<Self as frame_support::pallet_prelude::GenesisBuild<T>>::build_storage(self)
	}

	pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
		<Self as frame_support::pallet_prelude::GenesisBuild<T>>::assimilate_storage(self, storage)
	}
}
