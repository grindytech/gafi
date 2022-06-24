#![warn(missing_docs)]

fn main() -> sc_cli::Result<()> {

	#[cfg(feature = "with-gari")]
	return gafi_cli::command::run();

	#[cfg(feature = "with-gaki")]
	return gafi_cli::command::run();
	
	#[cfg(feature = "with-dev")]
	return gafi_local::command::run();
}
