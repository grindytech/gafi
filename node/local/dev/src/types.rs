/// Available Sealing methods.
#[cfg(feature = "manual-seal")]
#[derive(Debug, Copy, Clone, clap::ArgEnum)]
pub enum Sealing {
	// Seal using rpc method.
	Manual,
	// Seal when transaction is executed.
	Instant,
}

#[cfg(feature = "manual-seal")]
impl Default for Sealing {
	fn default() -> Sealing {
		Sealing::Manual
	}
}