use devnet::{
	AccountId, AuraConfig, Balance, BalancesConfig, EVMConfig,
	EthereumConfig, GenesisConfig, GrandpaConfig, UpfrontPoolConfig,
	StakingPoolConfig, Signature, SudoConfig, SystemConfig,
	AddressMappingConfig, FaucetConfig, TxHandlerConfig,
	WASM_BINARY, PoolConfig,
};
use gafi_primitives::{pool::{Level, Service, TicketType}};
use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public, H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::{collections::BTreeMap, str::FromStr};
use serde_json::json;
use gafi_primitives::{currency::{NativeToken::GAKI, unit, centi, GafiCurrency, TokenInfo}};
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

pub fn gaki_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../../resources/gakiTestnetSpecRaw.json")[..])
}

pub fn gaki_dev_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	let mut props : Properties = Properties::new();
	let aux = GafiCurrency::token_info(GAKI);
	let symbol = json!( String::from_utf8(aux.symbol).unwrap_or("GAKI".to_string()));
	let name  =json!( String::from_utf8(aux.name).unwrap_or("Gafi Network Kusama".to_string()));
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
			gaki_testnet_genesis(
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

/// Configure initial storage state for FRAME modules.
fn gaki_testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	// Pool config
	const MAX_PLAYER: u32 = 1000;
	let upfront_services = [
		(Level::Basic, Service::new(TicketType::Upfront(Level::Basic))),
		(Level::Medium, Service::new(TicketType::Upfront(Level::Medium))),
		(Level::Advance, Service::new(TicketType::Upfront(Level::Advance))),
	];
	let staking_services = [
		(Level::Basic, Service::new(TicketType::Staking(Level::Basic))),
		(Level::Medium, Service::new(TicketType::Staking(Level::Medium))),
		(Level::Advance, Service::new(TicketType::Staking(Level::Advance))),
	];
	const TIME_SERVICE: u128 = 60 * 60_000u128; // 1 hour
	let bond_existential_deposit: u128 = unit(GAKI);

	// pallet-faucet
	let faucet_amount: u128 = 1500 * unit(GAKI);

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// each genesis account hold 1M GAKI token
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1_000_000 * unit(GAKI))).collect(),
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
						"4e9A2Eee2caF9096161f9A5c3F0b0DE8f648AA11" //base
					)),
					pallet_evm::GenesisAccount {
						nonce: U256::zero(),
						balance: U256::from(1000 * unit(GAKI)),
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
		address_mapping: AddressMappingConfig {bond_deposit: bond_existential_deposit},
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
			gas_price: U256::from(100_000_000_000u128),
		},
		pool: PoolConfig {
			time_service: TIME_SERVICE,
		},
	}
}