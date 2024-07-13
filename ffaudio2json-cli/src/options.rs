use ffaudio2json::Channel;
use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[structopt(about = env!("CARGO_PKG_DESCRIPTION"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
pub struct Options {
	#[structopt(short, long, default_value = "800", help = "Number of samples to generate")]
	pub samples: u32,

	#[structopt(
		long,
		default_value = "-48",
		help = "Minimum value of the signal in dB that will be visible in the waveform"
	)]
	pub db_min: f64,

	#[structopt(
		long,
		default_value = "-48",
		help = "Maximum value of the signal in dB that will be visible in the waveform. Useful,if you know that your signal peaks at a certain level"
	)]
	pub db_max: f64,

	#[structopt(
		long,
		short,
		default_value = "false",
		help = "Use logarithmic (e.g. decibel) scale instead of linear scale"
	)]
	pub db_scale: bool,

	#[structopt(
		long,
		short,
		default_value = "6",
		help = "Precision of the floats that are generated. Reduce for smaller sized files. Usually 2 should be sufficient"
	)]
	pub precision: usize,

	#[structopt(long, short, default_value = "false", help = "Do not include the version info banner in the output")]
	pub no_header: bool,

	#[structopt(short, long, help = "Name of output file, defaults to <name of inputfile>.json")]
	pub output: Option<PathBuf>,

	#[structopt(long, help = "Channels to compute: left, right, mid, side, min, max", default_value = "left right")]
	#[clap(value_parser, value_delimiter = ' ')]
	pub channels: Vec<Channel>,

	#[structopt(short, long, help = "Suppress all output", default_value = "false")]
	pub quiet: bool,

	pub input: PathBuf,
}
impl From<Options> for ffaudio2json::Config {
	fn from(val: Options) -> Self {
		ffaudio2json::Config {
			samples: val.samples,
			db_min: val.db_min,
			db_max: val.db_max,
			db_scale: val.db_scale,
			precision: val.precision,
			no_header: val.no_header,
			output: val.output,
			input: val.input,
			channels: val.channels.into_iter().map(Into::into).collect(),
		}
	}
}
