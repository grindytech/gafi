use aurora_testnet_runtime::{
	pallet_pool::pool::PackService, AccountId, AuraConfig, Balance, BalancesConfig, EVMConfig,
	EthereumConfig, GenesisConfig, GrandpaConfig, PoolConfig, Signature, SudoConfig, SystemConfig,
	WASM_BINARY,
};
use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public, H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::{collections::BTreeMap, str::FromStr};
use serde_json::json;
use aurora_primitives::{AuroraNetworkCurrency, unit, currency::{NativeToken::AUX, TokenInfo}, centi};
use sp_std::*;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

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
    
	let mut props : Properties = Properties::new();
	let aux = AuroraNetworkCurrency::token_info(AUX);
	let symbol = json!( String::from_utf8(aux.symbol).unwrap_or("AUX".to_string()));
	let name  =json!( String::from_utf8(aux.name).unwrap_or("Aurora X".to_string()));
	let decimals  =json!(aux.decimals);
    props.insert("tokenSymbol".to_string(), symbol); 
    props.insert("tokenName".to_string(), name); 
	props.insert("tokenDecimals".to_string(), decimals);

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
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
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
		Some(props),
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
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
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

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	// Pool config
	let pool_fee: Balance =  75 * centi(AUX); // 0.75 AUX
	const MAX_PLAYER: u32 = 1000;
	let services: [(PackService, u8, u8, Balance); 3] = [
		(PackService::Basic, 4, 40, pool_fee),
		(PackService::Medium, 8, 70, pool_fee * 2),
		(PackService::Max, u8::MAX, 90, pool_fee * 3),
	];
	const TIME_SERVICE: u128 = 60 * 60_000u128; // 1 hour

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// each genesis account hold 1M AUX token
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1_000_000 * unit(AUX))).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				// H160 address of Alice dev account
				// Derived from SS58 (42 prefix) address
				// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
				// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
				// Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
				map.insert(
					H160::from_slice(&hex_literal::hex!(
						"d43593c715fdd31c61141abd04a99fd6822c8558" //base
					)),
					pallet_evm::GenesisAccount {
						nonce: U256::zero(),
						balance: U256::from(1000 * unit(AUX)),
						code: vec![],
						storage: std::collections::BTreeMap::new(),
					},
				);
				map.insert(
					H160::from_slice(&hex_literal::hex!(
						"b28049C6EE4F90AE804C70F860e55459E837E84b" //test 1
					)),
					pallet_evm::GenesisAccount {
						nonce: U256::zero(),
						balance: U256::from(1000 * unit(AUX)),
						code: vec![],
						storage: std::collections::BTreeMap::new(),
					},
				);
				map.insert(
					H160::from_slice(&hex_literal::hex!(
						"427491884a4baCA9a9a337e919f3aC96A0b88E64" //test 2
					)),
					pallet_evm::GenesisAccount {
						nonce: U256::zero(),
						balance: U256::from(1000 * unit(AUX)),
						code: vec![],
						storage: std::collections::BTreeMap::new(),
					},
				);
				map.insert(
					H160::from_slice(&hex_literal::hex!(
						"2f47229E4ef93ffdf8B6Fe7C5Df390C89e745314" //test 3
					)),
					pallet_evm::GenesisAccount {
						nonce: U256::zero(),
						balance: U256::from(1000 * unit(AUX)),
						code: vec![],
						storage: std::collections::BTreeMap::new(),
					},
				);
				map
			},
		},
		ethereum: EthereumConfig {},
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
		pool: PoolConfig { max_player: MAX_PLAYER, services, time_service: TIME_SERVICE },
	}
}
