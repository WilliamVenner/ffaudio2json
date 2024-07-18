//! # FFAudio2JSON
//!
//! Convert audio files to JSON waveforms!
//!
//! Based on [wav2json](https://github.com/beschulz/wav2json)
//!
//! ## Building
//!
//! This library/binary can be built/compiled just like any normal Rust program with `cargo build` or `cargo build --release`
//! (for an optimized build), however, you will probably want to also read the
//! [ffmpeg-next build instructions](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building)
//! as it is a dependency with special build requirements.
//!
//! ## Example Output
//!
//! ```json
//! {
//!   "_generator":"ffaudio2json version 0.1.0 on x86_64-pc-windows-msvc (https://github.com/WilliamVenner/ffaudio2json)",
//!   "left":[0.947125,0.901331,0.766280,0.578968,0.744371,0.575110,0.624754,0.739100,0.534745,0.561727,0.565447,0.777101,0.633872,0.443988,0.451541],
//!   "right":[0.895935,0.869228,0.782387,0.583250,0.806690,0.592015,0.599639,0.731451,0.472213,0.571442,0.524964,0.792326,0.549566,0.507130,0.494696],
//!   "mid":[0.921530,0.746390,0.774334,0.494298,0.775531,0.508056,0.601378,0.735276,0.393787,0.566585,0.459236,0.784713,0.426951,0.439940,0.462662],
//!   "side":[0.218711,0.460376,0.446356,0.467093,0.535112,0.556382,0.327098,0.455026,0.365384,0.321987,0.514186,0.492502,0.398223,0.324473,0.365328],
//!   "min":[0.947125,0.869228,0.766280,0.567466,0.744371,0.592015,0.605790,0.731451,0.534745,0.571442,0.565447,0.792326,0.633872,0.492440,0.494696],
//!   "max":[0.895935,0.901331,0.782387,0.583250,0.806690,0.537654,0.624754,0.739100,0.465301,0.563098,0.524964,0.777101,0.549566,0.507130,0.475655],
//!   "duration":168.552
//! }
//! ```

#![warn(missing_docs)]

extern crate ffmpeg_next as ffmpeg;

use crate::{
	channels::{ChannelWriter, Channels},
	generator::GeneratorContext,
};
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

#[doc(hidden)]
pub use strum::{IntoEnumIterator, VariantArray};

mod audio;
mod buffer;
mod generator;
mod util;

const JSON_HEADER: &str = concat!(
	"\n  \"_generator\":\"ffaudio2json version ",
	env!("CARGO_PKG_VERSION"),
	" on ",
	env!("TARGET_PLATFORM"),
	" (https://github.com/WilliamVenner/ffaudio2json)\","
);

impl FfAudio2Json {
	/// Generate the JSON waveform.
	///
	/// Returns the path to the output file.
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
			.or_else(|| {
				(|| {
					let mut ictx = ffmpeg::format::input(&self.input)?;
					// Seek to the last frame
					ictx.seek(i64::MAX, 0..i64::MAX)?;
					let pts = ictx.packets().last().and_then(|(_, packet)| packet.pts());
					ictx.seek(0, 0..i64::MAX)?;
					Ok::<_, Error>(pts)
				})()
				.expect("failed to seek to last frame in order to determine stream duration")
			})
			.expect("unable to determine stream duration") as f64
			* f64::from(stream.time_base());

		let input_samples = input_duration * decoder.rate() as f64;
		let resample_rate = {
			let dst_sample_rate = input_duration / (self.samples as f64).min(input_samples);
			(dst_sample_rate * decoder.rate() as f64) as usize
		};

		let writers = self.writers(&mut output, &output_path, input_samples.ceil() as usize)?;

		log::debug!(
			"Audio duration: {:?} ({} samples)",
			Duration::from_secs_f64(input_duration),
			input_samples,
		);

		log::debug!(
			"Codec: {} Channel(s): {} Format: {:?} Sample Rate: {} Hz",
			codec.description(),
			decoder.channels(),
			decoder.format(),
			decoder.rate()
		);

		log::debug!("Generating waveform...",);

		output.flush()?;
		GeneratorContext::generate(
			GeneratorContext {
				writers,
				buffer_capacity: resample_rate,
				config: &self,
				stream_idx,
			},
			&mut ictx,
			&mut decoder,
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

	fn writers(&self, output: &mut (impl Write + Seek), output_path: &Path, input_samples: usize) -> Result<Channels<ChannelWriter>, Error> {
		let mut writers = Channels::<ChannelWriter>::default();

		let samples_width = ((self.samples as usize).min(input_samples) * (self.precision + 3)) - 1;

		self.channels.iter().copied().try_for_each(|channel| {
			write!(output, "\n  \"{channel}\":[")?;

			let writer = ChannelWriter::new({
				let mut writer = self.open_output_file_writer(output_path)?;
				writer.seek(SeekFrom::Start(output.stream_position()?))?;
				BufWriter::new(writer)
			});

			match channel {
				Channel::Left => writers.left = Some(writer),
				Channel::Right => writers.right = Some(writer),
				Channel::Mid => writers.mid = Some(writer),
				Channel::Side => writers.side = Some(writer),
				Channel::Min => writers.min = Some(writer),
				Channel::Max => writers.max = Some(writer),
			}

			write!(output, "{:samples_width$}],", ' ', samples_width = samples_width)?;

			Ok::<_, std::io::Error>(())
		})?;

		Ok(writers)
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
