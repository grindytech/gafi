pub use sc_service::ChainSpec;

pub mod gari;

/// Can be called for a `Configuration` to identify which network the configuration targets.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Gari` network.
	fn is_gari(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_gari(&self) -> bool {
		self.id().starts_with("gari") || self.id().starts_with("gari")
	}
}
