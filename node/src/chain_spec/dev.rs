use devnet::{
	AccountId, AuraConfig, BalancesConfig, EVMConfig,
	EthereumConfig, GenesisConfig, GrandpaConfig, UpfrontPoolConfig,
	StakingPoolConfig, Signature, SudoConfig, SystemConfig,
	FaucetConfig, TxHandlerConfig,
	WASM_BINARY, PoolConfig, PalletCacheConfig, PalletCacheFaucetConfig
};
use gafi_primitives::currency::{unit, GafiCurrency, NativeToken::GAKI, TokenInfo};
use gafi_primitives::pool::{FlexService, Level};
use sc_service::{ChainType, Properties};
use serde_json::json;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public, H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{traits::{IdentifyAccount, Verify}, Permill};
use sp_std::*;
use std::collections::BTreeMap;

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

	let mut props: Properties = Properties::new();
	let gaki = GafiCurrency::token_info(GAKI);
	let symbol = json!(String::from_utf8(gaki.symbol).unwrap_or("GAKI".to_string()));
	let name = json!(String::from_utf8(gaki.name).unwrap_or("GAKI Token".to_string()));
	let decimals = json!(gaki.decimals);
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
			dev_genesis(
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
			dev_genesis(
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
fn dev_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	// Pool config
	const MAX_PLAYER: u32 = 1000;
	let upfront_services = [
		(
			Level::Basic,
			FlexService::new(10_u32, Permill::from_percent(30), 5 * unit(GAKI)),
		),
		(
			Level::Medium,
			FlexService::new(10_u32, Permill::from_percent(50), 7 * unit(GAKI)),
		),
		(
			Level::Advance,
			FlexService::new(10_u32, Permill::from_percent(70), 10 * unit(GAKI)),
		),
	];
	let staking_services = [
		(
			Level::Basic,
			FlexService::new(10_u32, Permill::from_percent(30), 1000 * unit(GAKI)),
		),
		(
			Level::Medium,
			FlexService::new(10_u32, Permill::from_percent(50), 1500 * unit(GAKI)),
		),
		(
			Level::Advance,
			FlexService::new(10_u32, Permill::from_percent(70), 2000 * unit(GAKI)),
		),
	];
	const TIME_SERVICE: u128 = 10 * 60_000u128; // for testing
	let bond_existential_deposit: u128 = unit(GAKI);
	let min_gas_price: U256 = U256::from(4_000_000_000_000u128);

	// pallet-faucet
	let faucet_amount: u128 = 1500 * unit(GAKI);

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// each genesis account hold 1M GAKI token
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1_000_000 * unit(GAKI)))
				.collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities
				.iter()
				.map(|x| (x.1.clone(), 1))
				.collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(
					H160::from_slice(&hex_literal::hex!(
						"4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11" //base
					)),
					fp_evm::GenesisAccount {
						nonce: U256::zero(),
						balance: U256::from(1_000_000 * unit(GAKI)),
						code: vec![],
						storage: std::collections::BTreeMap::new(),
					},
				);

				map.insert(
					H160::from_slice(&hex_literal::hex!(
						"F0B9EaA0fAaC58d5d4F3224958D75a5370672231"
					)),
					fp_evm::GenesisAccount {
						nonce: U256::zero(),
						balance: U256::from(1_000_000 * unit(GAKI)),
						code: vec![],
						storage: std::collections::BTreeMap::new(),
					},
				);

				map.insert(
					H160::from_slice(&hex_literal::hex!(
						"D910E83396231988F79df2f1175a90e15d26aB71"
					)),
					fp_evm::GenesisAccount {
						nonce: U256::zero(),
						balance: U256::from(1_000_000 * unit(GAKI)),
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
		upfront_pool: UpfrontPoolConfig { max_player: MAX_PLAYER, services: upfront_services },
		staking_pool: StakingPoolConfig { services: staking_services },
		faucet: FaucetConfig {
			genesis_accounts: vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			],
			faucet_amount,
		},
		tx_handler: TxHandlerConfig {
			gas_price: U256::from(min_gas_price),
		},
		pool: PoolConfig {
			time_service: TIME_SERVICE,
		},
		pallet_cache: PalletCacheConfig {
			clean_time: TIME_SERVICE,
			phantom: Default::default(),
			phantom_i: Default::default(),
		},
		pallet_cache_faucet: PalletCacheFaucetConfig {
			clean_time: TIME_SERVICE,
			phantom: Default::default(),
			phantom_i: Default::default(),
		},
	}
}
