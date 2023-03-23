use frame_support::{parameter_types, traits::LockIdentifier};
use gafi_primitives::currency::{centi, deposit, NativeToken::GAKI};
use runtime_common::prod_or_fast;
use static_assertions::const_assert;

use crate::{
	Balance, Balances, BlockNumber, Council, CouncilMaxMembers, RuntimeEvent, Runtime, Treasury, HOURS,
	MINUTES,
};

parameter_types! {
	pub CandidacyBond: Balance = 100 * centi(GAKI);
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub VotingBondBase: Balance = deposit(1, 64, GAKI);
	// additional data per vote is 32 bytes (account id).
	pub VotingBondFactor: Balance = deposit(0, 32, GAKI);
	/// Daily council elections
	pub TermDuration: BlockNumber = prod_or_fast!(24 * HOURS, 2 * MINUTES, "GAKI_TERM_DURATION");
	pub const DesiredMembers: u32 = 19;
	pub const DesiredRunnersUp: u32 = 19;
	pub const PhragmenElectionPalletId: LockIdentifier = *b"phrelect";
	pub const MaxCandidates: u32 = 10;
	pub const MaxVoters: u32 = 100;
}

// Make sure that there are no more than MaxMembers members elected via phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ChangeMembers = Council;
	type InitializeMembers = Council;
	type CurrencyToVote = frame_support::traits::U128CurrencyToVote;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type LoserCandidate = Treasury;
	type KickedMember = Treasury;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type PalletId = PhragmenElectionPalletId;
	type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
	type MaxCandidates = MaxCandidates;
	type MaxVoters = MaxVoters;
}
