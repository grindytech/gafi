#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use codec::{Decode, Encode};
use frame_support::{ensure, pallet_prelude::*, traits::Randomness, PalletId, RuntimeDebug};
use frame_system::{
	offchain::{
		AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
		SigningTypes, SubmitTransaction,
	},
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

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"gafi");
pub const UNSIGNED_TXS_PRIORITY: u64 = 10;

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

// /// Payload used by this example crate to hold price
// /// data required to submit a transaction.
// #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
// pub struct SeedPayload<Public, BlockNumber> {
// 	pub block_number: BlockNumber,
// 	pub seed: [u8; 32],
// 	pub public: Public,
// }

// impl<T: SigningTypes> SignedPayload<T> for SeedPayload<T::Public, BlockNumberFor<T>> {
// 	fn public(&self) -> T::Public {
// 		self.public.clone()
// 	}
// }

enum TransactionType {
	Signed,
	UnsignedForAny,
	UnsignedForAll,
	Raw,
	None,
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

		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// generate random ID
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple pallets send unsigned transactions.
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;

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

	/// Storing random seed generated from the off-chain worker every block
	#[pallet::storage]
	pub(crate) type RandomSeed<T: Config> = StorageValue<_, [u8; 32], ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored { something: u32, who: T::AccountId },
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
		/// Submit random seed from offchain-worker to runtime.
		///
		/// Only called by offchain-worker.
		///
		/// Arguments:
		/// - `seed`: random seed value.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(12)]
		#[pallet::weight({0})]
		pub fn submit_random_seed_unsigned(
			origin: OriginFor<T>,
			block_number: BlockNumberFor<T>,
			seed: [u8; 32],
		) -> DispatchResult {
			ensure_none(origin)?;
			// let block_number = <frame_system::Pallet<T>>::block_number();
			RandomSeed::<T>::set(seed);
			NextUnsignedAt::<T>::set(block_number);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn submit_random_seed_raw_unsigned(
			block_number: T::BlockNumber,
		) -> Result<(), &'static str> {
			let seed = sp_io::offchain::random_seed();
			log::info!("random_seed sp_io {:?}", seed);

			// let (_, result) = Signer::<T, T::AuthorityId>::any_account()
			// 	.send_unsigned_transaction(
			// 		|account| SeedPayload {
			// 			seed,
			// 			block_number,
			// 			public: account.public.clone(),
			// 		},
			// 		|payload, signature| Call::submit_random_seed_unsigned {
			// 			seed_payload: payload,
			// 			signature,
			// 		},
			// 	)
			// 	.ok_or("No local accounts accounts available.")?;
			let call = Call::submit_random_seed_unsigned { block_number, seed };

			let _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
			.map_err(|_| {
				log::error!("Failed in offchain_unsigned_tx");
			});

			Ok(())
		}

		fn validate_transaction_parameters(
			block_number: &BlockNumberFor<T>,
			seed: &[u8; 32],
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

			// // We prioritize transactions that are more far away from current average.
			// //
			// // Note this doesn't make much sense when building an actual oracle, but this example
			// // is here mostly to show off offchain workers capabilities, not about building an
			// // oracle.
			// let avg_price = Self::average_price()
			// 	.map(|price| {
			// 		if &price > new_price {
			// 			price - new_price
			// 		} else {
			// 			new_price - price
			// 		}
			// 	})
			// 	.unwrap_or(0);

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
				// The transaction is only valid for next 5 blocks. After that it's
				// going to be revalidated by the pool.
				.longevity(5)
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
		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			match call {
				Call::submit_random_seed_unsigned {
					block_number: _,
					seed: _,
				} => match source {
					TransactionSource::Local | TransactionSource::InBlock => {
						let valid_tx = |provide| {
							ValidTransaction::with_tag_prefix("game-randomness")
								.priority(UNSIGNED_TXS_PRIORITY) // please define `UNSIGNED_TXS_PRIORITY` before this line
								.and_provides([&provide])
								.longevity(3)
								.propagate(true)
								.build()
						};
						valid_tx(b"submit_random_seed_unsigned".to_vec())
					},
					_ => InvalidTransaction::Call.into(),
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}
}
