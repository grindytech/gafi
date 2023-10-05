use devnet_runtime::{
	AccountId, AuraConfig, BalancesConfig, FaucetConfig, GrandpaConfig, OracleRandomnessConfig,
	RuntimeGenesisConfig, Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use gafi_support::common::{unit, NativeToken::GAFI};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
						1_000_000_u128 * unit(GAFI),
					),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					authority_keys_from_seed("Alice"),
					authority_keys_from_seed("Bob"),
				],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Dave"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Eve"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Ferdie"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
						1_000_000_u128 * unit(GAFI),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
						1_000_000_u128 * unit(GAFI),
					),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, u128)>,
	_enable_println: bool,
) -> RuntimeGenesisConfig {
	RuntimeGenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			..Default::default()
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|(k, balance)| (k, balance)).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
			..Default::default()
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		faucet: FaucetConfig {
			genesis_accounts: endowed_accounts.iter().map(|x| (x.0.clone())).collect(),
		},
		oracle_randomness: OracleRandomnessConfig {
			default_urls: [
				"https://api.drand.sh/public/latest",
				"https://api2.drand.sh/public/latest",
				"https://api3.drand.sh/public/latest",
				"https://drand.cloudflare.com/public/latest",
			]
			.iter()
			.map(|s| s.as_bytes().to_vec())
			.collect(),
			
			..Default::default()
		},
	}
}
