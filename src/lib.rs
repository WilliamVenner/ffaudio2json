//! # ffaudio2json
//!
//! Convert audio files to JSON waveforms!
//!
//! Based on [wav2json](https://github.com/beschulz/wav2json)

#![warn(missing_docs)]

extern crate ffmpeg_next as ffmpeg;

use std::{
	borrow::Cow,
	ffi::OsStr,
	fs::{File, OpenOptions},
	io::{BufWriter, Seek, SeekFrom, Write},
	path::Path,
};
use strum::EnumCount;

mod error;
pub use error::Error;

mod config;
pub use config::*;

mod channels;
pub use channels::Channel;

mod audio;
mod util;

const JSON_HEADER: &str = concat!(
	"\n  \"_generator\":\"ffaudio2json version ",
	env!("CARGO_PKG_VERSION"),
	" on ",
	env!("TARGET_PLATFORM"),
	" (https://github.com/WilliamVenner/ffaudio2json)\","
);

struct GeneratorContext<'a> {
	channel_writers: [Option<ChannelWriter>; Channel::COUNT],
	config: &'a Config,
}

struct ChannelWriter {
	inner: BufWriter<File>,
	first_sample: bool,
}
impl ChannelWriter {
	fn write(&mut self, sample: f64, config: &Config) -> Result<(), std::io::Error> {
		if !core::mem::replace(&mut self.first_sample, false) {
			write!(self.inner, ",")?;
		}

		write!(self.inner, "{sample:.precision$}", precision = config.precision)?;

		Ok(())
	}
}

impl Config {
	/// Generate the JSON waveform
	pub fn run(self) -> Result<(), Error> {
		let output_path = self.output_file_path();
		let mut output = BufWriter::new(File::create(&output_path)?);

		output.write_all(b"{")?;

		if !self.no_header {
			output.write_all(JSON_HEADER.as_bytes())?;
		}

		ffmpeg::init()?;

		let mut ictx = ffmpeg::format::input(&self.input)?;

		let stream = ictx.streams().best(ffmpeg::media::Type::Audio).ok_or(ffmpeg::Error::StreamNotFound)?;

		let stream_idx = stream.index();

		let codec = ffmpeg::codec::decoder::find(stream.parameters().id())
			.ok_or(ffmpeg::Error::DecoderNotFound)?
			.audio()?;

		let mut decoder = ffmpeg::codec::Context::from_parameters(stream.parameters())?
			.decoder()
			.open_as(codec)?
			.audio()?;

		let input_duration = stream.duration() as f64 * f64::from(stream.time_base());
		let input_samples = input_duration * decoder.rate() as f64;
		let dst_sample_rate = input_duration / (self.samples as f64).min(input_samples);
		let resample_rate = (dst_sample_rate * decoder.rate() as f64) as usize;

		let samples_width = (self.samples as usize * (self.precision + 3)) - 1;
		let mut channel_writers = std::array::from_fn::<Option<ChannelWriter>, { Channel::COUNT }, _>(|_| None);
		for (channel, writer) in self.channels.iter().zip(channel_writers.iter_mut()) {
			write!(output, "\n  \"{channel}\":[")?;

			*writer = Some(ChannelWriter {
				inner: {
					let mut writer = self.open_output_file_writer(&output_path)?;
					writer.seek(SeekFrom::Start(output.stream_position()?))?;
					BufWriter::new(writer)
				},
				first_sample: true,
			});

			write!(output, "{:samples_width$}],", ' ', samples_width = samples_width)?;
		}
		output.flush()?;

		println!("Generating waveform...");
		println!(
			"Codec: {} Channel(s): {} Samples: {:?}",
			codec.description(),
			decoder.channels(),
			decoder.format()
		);

		audio::process(
			&mut ictx,
			&mut decoder,
			stream_idx,
			resample_rate,
			GeneratorContext {
				channel_writers,
				config: &self,
			},
		)?;
		output.flush()?;

		write!(output, "\n  \"duration\":{input_duration}\n}}")?;
		output.flush()?;

		Ok(())
	}

	fn output_file_path(&self) -> Cow<'_, Path> {
		self.output.as_deref().map(Cow::Borrowed).unwrap_or_else(|| {
			let mut file_name = self.input.file_name().unwrap_or(OsStr::new("output")).to_os_string();

			file_name.push(OsStr::new(".json"));

			Cow::Owned(self.input.with_file_name(file_name))
		})
	}

	fn open_output_file_writer(&self, path: &Path) -> Result<File, std::io::Error> {
		OpenOptions::new()
			.create(false)
			.create_new(false)
			.write(true)
			.truncate(false)
			.append(false)
			.read(false)
			.open(path)
	}
}
