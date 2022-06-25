use sc_chain_spec::ChainSpecExtension;
pub use sc_service::ChainSpec;
use serde::{Deserialize, Serialize};

pub mod gari;
use gari::*;

pub mod gaki;
use gaki::*;

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
	/// Known bad block hashes.
	#[serde(default)]
	pub bad_blocks: sc_client_api::BadBlocks<polkadot_primitives::v2::Block>,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

/// Can be called for a `Configuration` to identify which network the configuration targets.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Gari` network.
	fn is_gari(&self) -> bool;

	/// Returns if this is a configuration for the `Gaki` network.
	fn is_gaki(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_gaki(&self) -> bool {
		self.id().starts_with("gaki") || self.id().starts_with("gaki")
	}
	fn is_gari(&self) -> bool {
		self.id().starts_with("gari") || self.id().starts_with("gari")
	}
}
