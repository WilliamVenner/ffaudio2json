mod options;

use clap::Parser;
use options::Options;

fn main() -> Result<(), ffaudio2json::Error> {
	let opt = Options::parse();

	stderrlog::new()
		.module("ffaudio2json")
		.verbosity(log::Level::Debug)
		.timestamp(stderrlog::Timestamp::Millisecond)
		.quiet(opt.quiet)
		.init()
		.ok();

	let output_path = ffaudio2json::Config::from(opt).run()?;

	println!("{}", output_path.display());

	Ok(())
}
