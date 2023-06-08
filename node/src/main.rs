// #![warn(missing_docs)]

fn main() -> sc_cli::Result<()> {
	// #[cfg(feature = "with-gari")]
	// return gafi_cli::command::run();

	// #[cfg(feature = "with-dev")]
	// return solochain::command::run();

	// #[cfg(feature = "manual-seal")]
	// return solochain::command::run();

	#[cfg(feature = "with-game3")]
	return game3_node::command::run();

	#[cfg(feature = "runtime-benchmarks")]
	return game3_node::command::run();
}
