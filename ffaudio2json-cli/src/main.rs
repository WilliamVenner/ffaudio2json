mod options;

use clap::Parser;
use options::Options;

fn main() -> Result<(), ffaudio2json::Error> {
	ffaudio2json::Config::from(Options::parse()).run()
}
