// #![warn(missing_docs)]

fn main() -> sc_cli::Result<()> {

	#[cfg(feature = "with-gari")]
	return gafi_cli::command::run_gari();

	#[cfg(feature = "with-dev")]
	return gafi_local::command::run();

	#[cfg(feature = "manual-seal")]
	return gafi_local::command::run();

	// #[cfg(feature = "runtime-benchmarks")]
	// return gafi_cli::command::run_gari();
	
	// Devnet
	#[cfg(feature = "runtime-benchmarks")]
	return gafi_local::command::run();
}
