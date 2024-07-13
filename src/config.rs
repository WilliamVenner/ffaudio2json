//! Configuration for the behaviour of FfAudio2Json

use std::path::PathBuf;

#[derive(Debug, derive_builder::Builder)]
/// Configuration for the behaviour of FfAudio2Json
pub struct Config {
	/// Number of samples to generate
	pub samples: u32,

	/// Minimum value of the signal in dB that will be visible in the waveform. Useful,if you know that your signal peaks at a certain level
	pub db_min: f64,

	/// Maximum value of the signal in dB that will be visible in the waveform. Useful,if you know that your signal peaks at a certain level
	pub db_max: f64,

	/// Use logarithmic (e.g. decibel) scale instead of linear scale
	pub db_scale: bool,

	/// Precision of the floats that are generated. Reduce for smaller sized files. Usually 2 should be sufficient
	pub precision: usize,

	/// Omits the version info banner in the output
	pub no_header: bool,

	/// Name of output file, defaults to <name of inputfile>.json
	pub output: Option<PathBuf>,

	/// Channels to compute
	pub channels: Vec<Channel>,

	/// The path to the input audio file
	pub input: PathBuf,
}
impl Config {
	/// Create a new [`Config`] with the given input path
	pub fn new(input: impl Into<PathBuf>) -> Self {
		Self {
			input: input.into(),
			..Default::default()
		}
	}
}
impl Default for Config {
	fn default() -> Self {
		Self {
			samples: 800,
			db_min: -48.0,
			db_max: 0.0,
			db_scale: false,
			precision: 6,
			no_header: false,
			output: None,
			channels: vec![Channel::Left, Channel::Right],
			input: PathBuf::new(),
		}
	}
}

#[derive(Debug, Clone, Copy)]
/// The channels to compute
pub enum Channel {
	/// The left channel
	Left,

	/// The right channel
	Right,

	/// The mid channel
	Mid,

	/// The side channel
	Side,

	/// The min channel
	Min,

	/// The max channel
	Max,
}
