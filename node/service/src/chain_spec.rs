#![allow(unused_imports)]

use gafi_support::common::{unit, GafiCurrency, NativeToken::GAFI, TokenInfo};
use polkadot_core_primitives::AccountId;

use hex_literal::hex;
use sc_service::{ChainType, Properties};
use sc_telemetry::serde_json::json;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

#[cfg(feature = "devnet-native")]
use devnet_runtime::{
	AuraConfig, BalancesConfig, FaucetConfig, GrandpaConfig, OracleRandomnessConfig,
	RuntimeGenesisConfig, Signature, SudoConfig, SystemConfig, WASM_BINARY,
};

#[cfg(feature = "testnet-native")]
use testnet_runtime::{
	AuraConfig, BalancesConfig, FaucetConfig, GrandpaConfig, OracleRandomnessConfig,
	RuntimeGenesisConfig, Signature, SudoConfig, SystemConfig, WASM_BINARY,
};

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
			test_genesis(
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

pub fn local_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			test_genesis(
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

pub fn testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	let mut props: Properties = Properties::new();
	let token = GafiCurrency::token_info(GAFI);
	let symbol = json!(String::from_utf8(token.symbol).unwrap_or("GAFI".to_string()));
	let name = json!(String::from_utf8(token.name).unwrap_or("GAFI Token".to_string()));
	let decimals = json!(token.decimals);
	props.insert("tokenSymbol".to_string(), symbol);
	props.insert("tokenName".to_string(), name);
	props.insert("tokenDecimals".to_string(), decimals);

	Ok(ChainSpec::from_genesis(
		// Name
		"Testnet",
		// ID
		"gafi_testnet",
		ChainType::Live,
		move || {
			test_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![],
				// Sudo account
				hex!("6e312c24b61893b64fbb04fd37b2cc6c1df62a4419f1327fbabad172182a395c").into(),
				// Pre-funded accounts
				vec![],
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
		Some(props),
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
#[cfg(all(not(feature = "devnet-native"), not(feature = "testnet-native")))]
fn live_genesis(
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

/// Configure initial storage state for FRAME modules.
fn test_genesis(
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
