#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::{
	offchain::{CreateSignedTransaction, SubmitTransaction},
	pallet_prelude::*,
};
use gafi_support::game::{GameRandomness, SeedPayload};
use lite_json::json::JsonValue;
pub use pallet::*;
use sp_io::hashing::blake2_256;
use sp_runtime::{
	offchain::{http, Duration},
	traits::{Get, TrailingZeroInput},
	Saturating,
};
use sp_std::{marker::PhantomData, vec, vec::Vec};

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

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type RandomAttemps: Get<u32>;

		#[pallet::constant]
		type SeedLength: Get<u32>;

		#[pallet::constant]
		type MaxRandomURL: Get<u32>;

		#[pallet::constant]
		type RandomURLLength: Get<u32>;

		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple pallets send unsigned transactions.
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;

		/// Number of blocks of cooldown after unsigned transaction is included.
		///
		/// This ensures that we only accept unsigned transactions once, every `UnsignedInterval`
		/// blocks.
		#[pallet::constant]
		type UnsignedInterval: Get<BlockNumberFor<Self>>;
	}

	/// Storing random seed generated.
	#[pallet::storage]
	pub(crate) type RandomSeed<T: Config> =
		StorageValue<_, SeedPayload<BlockNumberFor<T>, BoundedVec<u8, T::SeedLength>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn urls)]
	pub type RandomnessURLs<T: Config> = StorageValue<
		_,
		BoundedVec<BoundedVec<u8, T::RandomURLLength>, T::MaxRandomURL>,
		ValueQuery,
	>;

	// #[pallet::type_value]
	// pub fn DefaultURLIndexing<T: Config>() -> u32 {
	// 	0u32
	// }

	// #[pallet::storage]
	// #[pallet::getter(fn url_indexing)]
	// pub type URLIndexing<T: Config> = StorageValue<
	// 	_,
	// 	u32,
	// 	ValueQuery,
	// 	DefaultURLIndexing<T>,
	// >;

	#[pallet::storage]
	#[pallet::getter(fn next_unsigned_at)]
	pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_randomness_url)]
	pub(super) type SelectedRandomnessURL<T: Config> =
		StorageValue<_, BoundedVec<u8, T::RandomURLLength>, OptionQuery>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T> {
		pub default_urls: Vec<Vec<u8>>,
		pub _phantom: PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			for url in &self.default_urls {
				if let Ok(source) = BoundedVec::<u8, T::RandomURLLength>::try_from(url.clone()) {
					let _ = <RandomnessURLs<T>>::try_append(source).map_or((), |_| {});
				}
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewOracleRandomnessURL { urls: Vec<Vec<u8>> },
		NewOracleRandomnessSeed { seed: Vec<u8> },
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidSeed,
		ExceedRandomURLLength,
		ExceedMaxRandomURL,
		InvalidPayload,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			let res = Self::fetch_random_and_send_raw_unsign(block_number);
			if let Err(e) = res {
				log::error!("Error: {}", e);
			} else {
				if let Some(seed_payload) = RandomSeed::<T>::get() {
					log::info!("Oracle Randomness: \n{:?}", seed_payload);
				} else {
					log::info!("Oracle Randomness: None");
				}
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set_new_random_urls())]
		pub fn set_new_random_urls(origin: OriginFor<T>, urls: Vec<Vec<u8>>) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				urls.len() as u32 <= T::MaxRandomURL::get(),
				Error::<T>::ExceedMaxRandomURL
			);

			let mut new_urls: Vec<BoundedVec<u8, T::RandomURLLength>> = vec![];
			for url in urls.clone() {
				let new_url = BoundedVec::<u8, T::RandomURLLength>::try_from(url);

				if let Ok(url_value) = new_url {
					new_urls.push(url_value);
				} else {
					return Err(Error::<T>::ExceedRandomURLLength.into())
				}
			}

			let new_random_url =
				BoundedVec::<BoundedVec<u8, T::RandomURLLength>, T::MaxRandomURL>::try_from(
					new_urls,
				);

			if let Ok(url_values) = new_random_url {
				RandomnessURLs::<T>::put(url_values)
			} else {
				return Err(Error::<T>::ExceedMaxRandomURL.into())
			}

			Self::deposit_event(Event::NewOracleRandomnessURL { urls });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::submit_random_seed_unsigned())]
		pub fn submit_random_seed_unsigned(
			origin: OriginFor<T>,
			block_number: BlockNumberFor<T>,
			seed: Vec<u8>,
		) -> DispatchResult {
			ensure_none(origin)?;
			ensure!(
				seed.len() as u32 == T::SeedLength::get(),
				Error::<T>::InvalidSeed
			);

			let bounded_seed = BoundedVec::<u8, T::SeedLength>::try_from(seed.clone());

			if let Ok(seed) = bounded_seed {
				let new_payload = SeedPayload { block_number, seed };
				RandomSeed::<T>::put(new_payload);
				<NextUnsignedAt<T>>::put(block_number.saturating_add(T::UnsignedInterval::get()));

				if let Some(next_url) = Self::get_next_url(
					RandomnessURLs::<T>::get()
						.into_iter()
						.map(|inner_vec| inner_vec.into_inner())
						.collect(),
					SelectedRandomnessURL::<T>::get().map(|bounded_vec| bounded_vec.into_inner()),
				) {
					if let Ok(url) = BoundedVec::<u8, T::RandomURLLength>::try_from(next_url) {
						SelectedRandomnessURL::<T>::put(url);
					}
				}
			} else {
				return Err(Error::<T>::InvalidPayload.into())
			}
			Self::deposit_event(Event::<T>::NewOracleRandomnessSeed { seed });
			return Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_next_url(
			urls: Vec<Vec<u8>>,
			maybe_current_url: Option<Vec<u8>>,
		) -> Option<Vec<u8>> {
			if let Some(current_url) = maybe_current_url {
				if let Some(index) = urls.iter().position(|url| url == &current_url) {
					let next_index = (index + 1) % urls.len();
					return Some(urls[next_index].clone())
				}
			}
			return urls.first().map(|v| v.clone())
		}
	}

	// Random implementation
	impl<T: Config> Pallet<T> {
		pub(crate) fn gen_random(seed: &[u8]) -> Result<u32, Error<T>> {
			match <u32>::decode(&mut TrailingZeroInput::new(seed.as_ref())) {
				Ok(random) => Ok(random),
				Err(_) => Err(Error::<T>::InvalidSeed),
			}
		}

		pub fn random_bias(seed: &[u8], total: u32, attempts: u32) -> Option<u32> {
			let mut random_number = Self::gen_random(&seed);

			for _ in 1..attempts {
				if let Ok(rand_val) = random_number {
					if rand_val < u32::MAX.saturating_sub(u32::MAX.wrapping_rem(total)) {
						break
					}
					random_number = Self::gen_random(&seed);
				}
			}
			if let Ok(rand_val) = random_number {
				return Some((rand_val.wrapping_rem(total)).saturating_add(1))
			}

			None
		}
	}

	impl<T: Config> GameRandomness for Pallet<T> {
		/// Generates a random number between 1 and `total` (inclusive).
		/// This function repeats the process up to `RandomAttemps` times if
		/// the number falls within the overflow range of the modulo operation to mitigate modulo
		/// bias.
		///
		/// Returns `None` if `total` is 0.
		fn random_number(total: u32, adjust: u32) -> Option<u32> {
			if total == 0 {
				return None
			}

			if let Some(payload) = RandomSeed::<T>::get() {
				let mut extended_seed = payload.seed.to_vec();
				extended_seed.extend_from_slice(&adjust.to_le_bytes());
				let seed_hash = blake2_256(&extended_seed);
				return Self::random_bias(&seed_hash, total, T::RandomAttemps::get())
			}
			None
		}
	}

	// Offchain implementation
	impl<T: Config> Pallet<T> {
		fn submit_random_seed_raw_unsigned(
			block_number: BlockNumberFor<T>,
			seed: Vec<u8>,
		) -> Result<(), &'static str> {
			let call = Call::submit_random_seed_unsigned { block_number, seed };
			let _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
				.map_err(|_| {
					log::error!("Failed in offchain_unsigned_tx");
				});
			Ok(())
		}

		pub fn fetch_random_and_send_raw_unsign(
			block_number: BlockNumberFor<T>,
		) -> Result<(), &'static str> {
			// try to fetch from currently selected url
			if let Some(url) = SelectedRandomnessURL::<T>::get() {
				if let Ok(url_str) = sp_std::str::from_utf8(&url) {
					let response = Self::fetch_random(url_str);
					if let Ok(randomness) = response {
						return Self::submit_random_seed_raw_unsigned(block_number, randomness)
					}
				}
			}

			// switch url
			for url in RandomnessURLs::<T>::get() {
				if let Ok(url_str) = sp_std::str::from_utf8(&url) {
					let response = Self::fetch_random(url_str);
					if let Ok(randomness) = response {
						return Self::submit_random_seed_raw_unsigned(block_number, randomness)
					}
				}
			}

			Err("Randomness URL not available")
		}

		pub(crate) fn parse_randomness(result: &str) -> Option<Vec<u8>> {
			let val = lite_json::parse_json(result);

			let randomness = match val.ok()? {
				JsonValue::Object(obj) => {
					let (_, v) = obj
						.into_iter()
						.find(|(k, _)| k.iter().copied().eq("randomness".chars()))?;
					match v {
						JsonValue::String(randomness) => randomness,
						_ => return None,
					}
				},
				_ => return None,
			};

			let randomness: Vec<char> = randomness.iter().copied().collect();
			let bytes = randomness.iter().map(|&c| c as u8).collect();
			Some(bytes)
		}

		pub fn fetch_random(url: &str) -> Result<Vec<u8>, http::Error> {
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));

			let request = http::Request::get(url);

			let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;

			let response =
				pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;

			if response.code != 200 {
				log::warn!("Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown)
			}

			let body = response.body().collect::<Vec<u8>>();

			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::warn!("No UTF8 body");
				http::Error::Unknown
			})?;

			let randomness = match Self::parse_randomness(body_str) {
				Some(random) => Ok(random),
				None => {
					log::warn!(
						"Unable to extract randomness from the response: {:?}",
						body_str
					);
					Err(http::Error::Unknown)
				},
			}?;

			Ok(randomness)
		}

		fn validate_transaction_parameters(
			block_number: &BlockNumberFor<T>,
			_seed: &Vec<u8>,
		) -> TransactionValidity {
			let next_unsigned_at = <NextUnsignedAt<T>>::get();
			if &next_unsigned_at > block_number {
				return InvalidTransaction::Stale.into()
			}

			// Let's make sure to reject transactions from the future.
			let current_block = <frame_system::Pallet<T>>::block_number();
			if &current_block < block_number {
				return InvalidTransaction::Future.into()
			}

			ValidTransaction::with_tag_prefix("oracle-randomness")
				// We set base priority to 2**20 and hope it's included before any other
				.priority(T::UnsignedPriority::get())
				// The transaction is only valid for next 5 blocks. After that it's
				// going to be revalidated by the pool.
				.longevity(1)
				// It's fine to propagate that transaction to other peers, which means it can be
				// created even by nodes that don't produce blocks.
				// Note that sometimes it's better to keep it for yourself (if you are the block
				// producer), since for instance in some schemes others may copy your solution and
				// claim a reward.
				.propagate(true)
				.build()
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::submit_random_seed_unsigned { block_number, seed } = call {
				Self::validate_transaction_parameters(block_number, seed)
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}
}
