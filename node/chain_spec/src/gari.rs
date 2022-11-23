use cumulus_primitives_core::ParaId;
use gafi_primitives::currency::{GafiCurrency, NativeToken::GAFI, TokenInfo};
use gari_runtime::{
	types::{AccountId, Signature, EXISTENTIAL_DEPOSIT},
	EVMConfig, FaucetConfig, SudoConfig,
};
use hex_literal::hex;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::collections::BTreeMap;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<gari_runtime::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Helper function to generate a crypto pair from seed
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_public_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_public_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(keys: AuraId) -> gari_runtime::SessionKeys {
	gari_runtime::SessionKeys { aura: keys }
}

pub fn rococo_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut props = sc_chain_spec::Properties::new();

	let gafi = GafiCurrency::token_info(GAFI);
	let symbol = json!(String::from_utf8(gafi.symbol).unwrap_or("GAFI".to_string()));
	let name = json!(String::from_utf8(gafi.name).unwrap_or("Gafi Token".to_string()));
	let decimals = json!(gafi.decimals);
	props.insert("tokenSymbol".to_string(), symbol);
	props.insert("tokenName".to_string(), name);
	props.insert("tokenDecimals".to_string(), decimals);
	let id: ParaId = 4015.into();

	ChainSpec::from_genesis(
		// Name
		"Gari",
		// ID
		"gafi_rococo",
		ChainType::Live,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						//5FbfK2cdph7eM6YNmwyoNxq1hWhsCf8Gq4yet1yduexLCyTe
						hex!("6e312c24b61893b64fbb04fd37b2cc6c1df62a4419f1327fbabad172182a395c")
							.into(),
						hex!("9c50bd296e31f84d7ff151faf176e8d4ff6894c8f55ad95fcbe69123aa9ded2d")
							.unchecked_into(),
					),
					(
						//5GRQ2cn1yjWg4KeC386X6kNHwLkQnxf1acushGeurqKnpUWb
						hex!("0253d36985ec33de94eadc8657dc5ab9dbfa84e27d944660c8c758a1530d2462")
							.into(),
						hex!("c0b91242dac16a951f8ca60e9d0c3937f6a01012a4d299b05b07e047009cbc57")
							.unchecked_into(),
					),
				],
				vec![
					//5FYu1DAUqRax7Pe8tQxQQccs1SyZNLn8oPaLgJo9RtaBge5o
					(
						hex!("9a3518c4346239d5384abd80659d358f23271e1049398dca23734203ab44b811")
							.into(),
						500_000_000_000_000_000_000_000_000_u128,
					),
					//5Gp4fsJUTCtKXfXwjsJvNevtJeZrKFGm3kvbrtWePqpCqtCZ
					(
						hex!("d2028d37ded894f5544d2c93efa810772e59f5340e169c54230d88ef1ef2ff1f")
							.into(),
						500_000_000_000_000_000_000_000_000_u128,
					),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				id,
			)
		},
		Vec::new(),
		None,
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

pub fn development_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Dave"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Eve"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Ferdie"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				1000.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: 1000,
		},
	)
}

pub fn local_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Gari Testnet",
		// ID
		"gari_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Dave"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Eve"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Ferdie"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
						500_000_000_000_000_000_000_000_000_u128,
					),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				2000.into(),
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("gari-testnet"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: 2000,
		},
	)
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<(AccountId, u128)>,
	root_key: AccountId,
	id: ParaId,
) -> gari_runtime::GenesisConfig {
	gari_runtime::GenesisConfig {
		system: gari_runtime::SystemConfig {
			code: gari_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: gari_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k.0, k.1)).collect(),
		},
		sudo: SudoConfig {
			key: Some(root_key),
		},
		parachain_info: gari_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: gari_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: gari_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						template_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: gari_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
		evm: EVMConfig {
			accounts: {
				let map = BTreeMap::new();
				map
			},
		},
		faucet: FaucetConfig {
			genesis_accounts: vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
			],
		},
	}
}
