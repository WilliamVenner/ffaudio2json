//! # ffaudio2json
//!
//! Convert audio files to JSON waveforms!
//!
//! Based on [wav2json](https://github.com/beschulz/wav2json)

#![warn(missing_docs)]

extern crate ffmpeg_next as ffmpeg;

use channels::{ChannelWriter, Channels};
use std::{
	borrow::Cow,
	ffi::OsStr,
	fs::{File, OpenOptions},
	io::{BufWriter, Seek, SeekFrom, Write},
	path::{Path, PathBuf},
	time::{Duration, Instant},
};

mod error;
pub use error::Error;

mod config;
pub use config::*;

mod channels;
pub use channels::Channel;
pub use strum::{IntoEnumIterator, VariantArray};

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
	buffer_capacity: usize,
	channels: Channels<ChannelWriter>,
	config: &'a Config,
}

impl Config {
	/// Generate the JSON waveform
	///
	/// Returns the path to the output file
	pub fn run(self) -> Result<PathBuf, Error> {
		let now = Instant::now();

		let input_file_size = self.input.metadata()?.len();

		let output_path = self.output_file_path();
		let mut output = BufWriter::new(File::create(&output_path)?);

		output.write_all(b"{")?;

		if !self.no_header {
			output.write_all(JSON_HEADER.as_bytes())?;
		}

		ffmpeg::init()?;

		let mut ictx = ffmpeg::format::input(&self.input)?;

		let pts = (|| {
			// Seek to the last frame
			ictx.seek(i64::MAX, 0..i64::MAX)?;
			let pts = ictx.packets().last().and_then(|(_, packet)| packet.pts());
			ictx.seek(0, 0..i64::MAX)?;
			Ok::<_, Error>(pts)
		})();

		let stream = ictx.streams().best(ffmpeg::media::Type::Audio).ok_or(ffmpeg::Error::StreamNotFound)?;

		let stream_idx = stream.index();

		let codec = ffmpeg::codec::decoder::find(stream.parameters().id())
			.ok_or(ffmpeg::Error::DecoderNotFound)?
			.audio()?;

		let mut decoder = ffmpeg::codec::Context::from_parameters(stream.parameters())?
			.decoder()
			.open_as(codec)?
			.audio()?;

		let input_duration = Some(stream.duration())
			.filter(|duration| *duration != i64::MIN)
			.or_else(|| pts.expect("failed to seek to last frame in order to determine stream duration"))
			.expect("unable to determine stream duration") as f64
			* f64::from(stream.time_base());

		log::debug!("Audio duration: {:?}", Duration::from_secs_f64(input_duration));

		let input_samples = input_duration * decoder.rate() as f64;
		let dst_sample_rate = input_duration / (self.samples as f64).min(input_samples);
		let resample_rate = (dst_sample_rate * decoder.rate() as f64) as usize;

		let mut channels = Channels::<ChannelWriter>::default();
		{
			let samples_width = (self.samples as usize * (self.precision + 3)) - 1;

			self.channels.iter().copied().try_for_each(|channel| {
				write!(output, "\n  \"{channel}\":[")?;

				let writer = ChannelWriter::new({
					let mut writer = self.open_output_file_writer(&output_path)?;
					writer.seek(SeekFrom::Start(output.stream_position()?))?;
					BufWriter::new(writer)
				});

				match channel {
					Channel::Left => channels.left = Some(writer),
					Channel::Right => channels.right = Some(writer),
					Channel::Mid => channels.mid = Some(writer),
					Channel::Side => channels.side = Some(writer),
					Channel::Min => channels.min = Some(writer),
					Channel::Max => channels.max = Some(writer),
				}

				write!(output, "{:samples_width$}],", ' ', samples_width = samples_width)?;

				Ok::<_, std::io::Error>(())
			})?;
		}

		output.flush()?;

		log::debug!(
			"Codec: {} Channel(s): {} Samples: {:?}",
			codec.description(),
			decoder.channels(),
			decoder.format()
		);
		log::debug!("Generating waveform...");

		audio::process(
			&mut ictx,
			&mut decoder,
			stream_idx,
			GeneratorContext {
				channels,
				buffer_capacity: resample_rate,
				config: &self,
			},
		)?;
		output.flush()?;

		write!(output, "\n  \"duration\":{input_duration}\n}}")?;
		output.flush()?;

		let elapsed = now.elapsed();
		log::debug!(
			"Took {:?} ({:.2} MiB/s) ({:?} of audio/s)",
			elapsed,
			input_file_size as f64 / elapsed.as_secs_f64() / 1024.0 / 1024.0,
			Duration::try_from_secs_f64(input_duration / elapsed.as_secs_f64()).unwrap_or(Duration::ZERO)
		);

		Ok(output_path.into_owned())
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

// TODO log
