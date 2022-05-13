#![cfg(feature = "runtime-benchmarks")]

use crate::*;
// #[allow(unused)]
use crate::Pallet as GameCreator;
use crate::{Call, Config};
use frame_benchmarking::Box;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::log::info;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use pallet_evm::AddressMapping;
use pallet_evm::Runner;
use pallet_evm::{ExitReason, ExitSucceed};
use scale_info::prelude::format;
use scale_info::prelude::string::String;

use sp_core::{H160, U256};
use sp_std::str::FromStr;

fn make_free_balance<T: Config>(acc: &T::AccountId, balance: u64) {
    let balance_amount = balance.try_into().ok().unwrap();
    <T as pallet::Config>::Currency::make_free_balance_be(acc, balance_amount);
    <T as pallet::Config>::Currency::issue(balance_amount);
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn new_funded_account<T: Config>(index: u32, seed: u32, balance: u64) -> T::AccountId {
    let name: String = format!("{}{}", index, seed);
    let user = account(string_to_static_str(name), index, seed);
    make_free_balance::<T>(&user, balance);
    return user;
}


benchmarks! {
    claim_contract {
    	let s in 0 .. 1;
        let evm_acc = H160::from_str("0x4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11").unwrap();
        let sub_acc = T::AddressMapping::into_account_id(evm_acc);
        make_free_balance::<T>(&sub_acc, 1000_000_000_000_u64);

    }: _(RawOrigin::Signed(sub_acc), evm_acc)
}

#[cfg(test)]
mod mock {

    pub use crate::{self as game_creator};
    use frame_support::{
        dispatch::Vec,
        traits::{Currency, OnFinalize, OnInitialize},
    };
    use frame_support::{parameter_types, traits::GenesisBuild};
    use frame_system as system;
    use gafi_primitives::currency::{unit, NativeToken::GAKI};
    pub use pallet_balances::Call as BalancesCall;
    use pallet_evm::{EVMCurrencyAdapter, EnsureAddressNever, EnsureAddressTruncated};
    use sp_core::{H256, U256};
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        AccountId32,
    };

    type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
    type Block = frame_system::mocking::MockBlock<Runtime>;

    frame_support::construct_runtime!(
        pub enum Runtime where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
            Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
            GameCreator: game_creator::{Pallet, Storage, Event<T>},
            ProofAddressMapping: proof_address_mapping::{Pallet, Storage, Event<T>},
            Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, Origin},
            EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
            Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const SS58Prefix: u8 = 24;
    }
    impl system::Config for Runtime {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type Origin = Origin;
        type Call = Call;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId32;
        type AccountData = pallet_balances::AccountData<u128>;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = Event;
        type BlockHashCount = BlockHashCount;
        type Version = ();
        type PalletInfo = PalletInfo;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = SS58Prefix;
        type OnSetCode = ();
        type MaxConsumers = frame_support::traits::ConstU32<16>;
    }

    impl pallet_ethereum::Config for Runtime {
        type Event = Event;
        type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
    }

    parameter_types! {
        pub const ChainId: u64 = 1337;
        pub BlockGasLimit: U256 = U256::from(u32::max_value());
    }
    impl pallet_evm::Config for Runtime {
        type FeeCalculator = ();
        type GasWeightMapping = ();
        type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
        type CallOrigin = EnsureAddressTruncated;
        type WithdrawOrigin = EnsureAddressNever<AccountId32>;
        type AddressMapping = ProofAddressMapping;
        type Currency = Balances;
        type Event = Event;
        type Runner = pallet_evm::runner::stack::Runner<Self>;
        type PrecompilesType = ();
        type PrecompilesValue = ();
        type ChainId = ChainId;
        type BlockGasLimit = BlockGasLimit;
        type OnChargeTransaction = EVMCurrencyAdapter<Balances, ()>;
        type FindAuthor = ();
    }

    parameter_types! {
        pub Prefix: &'static [u8] =  b"Bond Gafi Network account:";
        pub Fee: u128 = 1 *  unit(GAKI);
    }
    impl proof_address_mapping::Config for Runtime {
        type Event = Event;
        type Currency = Balances;
        type WeightInfo = ();
        type MessagePrefix = Prefix;
        type ReservationFee = Fee;
    }

    pub const EXISTENTIAL_DEPOSIT: u128 = 1000;

    parameter_types! {
        pub ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT;
    }

    impl pallet_balances::Config for Runtime {
        type MaxLocks = ();
        type MaxReserves = ();
        type ReserveIdentifier = [u8; 8];
        type Balance = u128;
        type Event = Event;
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type WeightInfo = ();
    }

    pub const GAME_CREATE_FEE: u128 = 1_000_0000u128;

    parameter_types! {
        pub MaxContractOwned: u32 = 100;
        pub GameCreatorFee: u128 = GAME_CREATE_FEE;
    }

    impl game_creator::Config for Runtime {
        type Event = Event;
        type Currency = Balances;
        type AddressMapping = ProofAddressMapping;
        type MaxContractOwned = MaxContractOwned;
        type ContractCreator = EVM;
        type ReservationFee = GameCreatorFee;
    }

    pub const MILLISECS_PER_BLOCK: u64 = 6000;
    pub const INIT_TIMESTAMP: u64 = 30_000;
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    parameter_types! {
        pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
    }

    impl pallet_timestamp::Config for Runtime {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
        type WeightInfo = ();
    }
}



// impl_benchmark_test_suite!(GameCreator, crate::mock::new_test_ext(), crate::mock::Test,);
