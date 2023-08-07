#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use codec::{Decode, Encode};
use frame_support::{pallet_prelude::*, traits::Randomness, PalletId};
use frame_system::{
	offchain::{CreateSignedTransaction, SubmitTransaction},
	pallet_prelude::BlockNumberFor,
};
use sp_core::{offchain::KeyTypeId, Get};
use sp_runtime::{
	traits::TrailingZeroInput,
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
};

mod features;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

pub type Seed = [u8; 32];

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"gafi");
pub const UNSIGNED_TXS_PRIORITY: u64 = 10;

pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);
	pub struct TestAuthId;

	// implemented for runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

/// Payload used to hold seed data required to submit a transaction.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo, MaxEncodedLen)]
pub struct SeedPayload<BlockNumber, Seed> {
	block_number: BlockNumber,
	seed: Seed,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		/// Generates low-influence random values.
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

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
		type UnsignedInterval: Get<Self::BlockNumber>;

		/// Number of attempts to re-randomize in order to reduce modulus bias.
		#[pallet::constant]
		type RandomAttemps: Get<u32>;
	}

	/// Defines the block when next unsigned transaction will be accepted.
	///
	/// To prevent spam of unsigned (and unpaid!) transactions on the network,
	/// we only allow one transaction every `T::UnsignedInterval` blocks.
	/// This storage entry defines when new transaction is going to be accepted.
	#[pallet::storage]
	#[pallet::getter(fn next_unsigned_at)]
	pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Storing random seed generated from the off-chain worker.
	#[pallet::storage]
	pub(crate) type RandomSeed<T: Config> =
		StorageValue<_, SeedPayload<BlockNumberFor<T>, Seed>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event generated when a new price is submitted.
		NewSeed {
			block_number: BlockNumberFor<T>,
			seed: Seed,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InvalidSeed,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			let res = Self::submit_random_seed_raw_unsigned(block_number);
			if let Err(e) = res {
				log::error!("Error: {}", e);
			}
			let random = RandomSeed::<T>::get();
			log::info!("random {:?}", random);
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submit a new random seed.
		///
		/// This function sets a new `seed` for randomness in every `T::UnsignedInterval` blocks.
		///
		/// # Parameters
		///
		/// - `origin`: Accepted only by the off-chain worker.
		/// - `block_number`: Current block number.
		/// - `seed`: New random seed.
		#[pallet::call_index(12)]
		#[pallet::weight({0})]
		pub fn submit_random_seed_unsigned(
			origin: OriginFor<T>,
			block_number: BlockNumberFor<T>,
			seed: Seed,
		) -> DispatchResult {
			ensure_none(origin)?;

			RandomSeed::<T>::put(SeedPayload { block_number, seed });
			<NextUnsignedAt<T>>::put(block_number + T::UnsignedInterval::get());
			Self::deposit_event(Event::<T>::NewSeed { block_number, seed });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Submits a random seed obtained from offchain source along with the provided block number
		/// for unsigned transaction.
		///
		/// # Arguments
		///
		/// - `block_number`: The block number to associate with the submitted random seed.
		fn submit_random_seed_raw_unsigned(
			block_number: T::BlockNumber,
		) -> Result<(), &'static str> {
			let seed = sp_io::offchain::random_seed();
			let call = Call::submit_random_seed_unsigned { block_number, seed };
			let _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
				.map_err(|_| {
					log::error!("Failed in offchain_unsigned_tx");
				});
			Ok(())
		}

		fn validate_transaction_parameters(
			block_number: &BlockNumberFor<T>,
			_seed: &Seed,
		) -> TransactionValidity {
			// Now let's check if the transaction has any chance to succeed.
			let next_unsigned_at = <NextUnsignedAt<T>>::get();
			if &next_unsigned_at > block_number {
				return InvalidTransaction::Stale.into()
			}
			// Let's make sure to reject transactions from the future.
			let current_block = <frame_system::Pallet<T>>::block_number();
			if &current_block < block_number {
				return InvalidTransaction::Future.into()
			}

			ValidTransaction::with_tag_prefix("game-randomness")
				// We set base priority to 2**20 and hope it's included before any other
				.priority(T::UnsignedPriority::get())
				// This transaction does not require anything else to go before into the pool.
				// In theory we could require `previous_unsigned_at` transaction to go first,
				// but it's not necessary in our case.
				//.and_requires()
				// We set the `provides` tag to be the same as `next_unsigned_at`. This makes
				// sure only one transaction produced after `next_unsigned_at` will ever
				// get to the transaction pool and will end up in the block.
				// We can still have multiple transactions compete for the same "spot",
				// and the one with higher priority will replace other one in the pool.
				.and_provides(next_unsigned_at)
				// The transaction is only valid for next 1 blocks. After that it's
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
