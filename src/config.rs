//! Configuration for the behaviour of FfAudio2Json

use crate::channels::Channel;
use std::path::PathBuf;

#[derive(Debug, derive_builder::Builder)]
#[builder(build_fn(validate = "Self::validate"))]
/// Configuration for the behaviour of FfAudio2Json
pub struct FfAudio2Json {
	/// Number of samples to generate
	#[builder(default = "800")]
	pub(crate) samples: u32,

	/// Minimum value of the signal in dB that will be visible in the waveform. Useful if you know that your signal peaks at a certain level
	#[builder(default = "-48.0")]
	pub(crate) db_min: f64,

	/// Maximum value of the signal in dB that will be visible in the waveform. Useful if you know that your signal peaks at a certain level
	#[builder(default = "0.0")]
	pub(crate) db_max: f64,

	/// Use logarithmic (e.g. decibel) scale instead of linear scale
	#[builder(default = "false")]
	pub(crate) db_scale: bool,

	/// Precision of the floats that are generated. Reduce for smaller sized files. Usually 2 should be sufficient
	#[builder(default = "6")]
	pub(crate) precision: usize,

	/// Omits the version info banner in the output
	#[builder(default = "false")]
	pub(crate) no_header: bool,

	/// Name of output file, defaults to `<name of inputfile>.json`
	#[builder(default = "None")]
	pub(crate) output: Option<PathBuf>,

	/// Channels to compute
	#[builder(default = "vec![Channel::Left, Channel::Right]")]
	pub(crate) channels: Vec<Channel>,

	/// The path to the input audio file
	pub(crate) input: PathBuf,
}
impl FfAudio2Json {
	/// Creates a new builder for [`FfAudio2Json`]
	pub fn builder() -> FfAudio2JsonBuilder {
		FfAudio2JsonBuilder::default()
	}
}

impl FfAudio2JsonBuilder {
	fn validate(&self) -> Result<(), String> {
		if self.channels.as_ref().is_some_and(|channels| channels.is_empty()) {
			return Err("At least one channel must be specified".to_string());
		}

		Ok(())
	}
}
