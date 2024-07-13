//! # ffaudio2json
//!
//! Convert audio files to JSON waveforms!
//!
//! Based on [wav2json](https://github.com/beschulz/wav2json)

#![warn(missing_docs)]

use ffmpeg_next as ffmpeg;
use std::{
	borrow::Cow,
	ffi::OsStr,
	fs::File,
	io::{BufWriter, Write},
};

mod error;
pub use error::Error;

mod config;
pub use config::*;

mod util;

const JSON_HEADER: &str = concat!(
	"  \"_generator\":\"ffaudio2json version ",
	env!("CARGO_PKG_VERSION"),
	" on ",
	env!("TARGET_PLATFORM"),
	" (https://github.com/WilliamVenner/ffaudio2json)\",\n"
);

impl Config {
	/// Generate the JSON waveform
	pub fn run(self) -> Result<(), Error> {
		let mut output = self.open_output_file()?;

		output.write_all(b"{\n")?;

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

		let mut buffer = Vec::with_capacity(resample_rate * 2);

		output.write_all(b"  \"mid\":[")?;

		let process_frame = |decoded: &ffmpeg::frame::Audio, buffer: &mut Vec<i16>| {
			match (decoded.format(), decoded.channels()) {
				(ffmpeg_next::format::Sample::I16(_), 1) => {
					let samples: &[i16] = decoded.plane(0);

					// TODO optimise
					buffer.extend_from_slice(samples);
				}

				_ => {
					return Err(Error::UnsupportedFormat {
						format: decoded.format(),
						channels: decoded.channels(),
					})
				}
			}
			Ok(())
		};

		for (_, packet) in ictx.packets().filter(|(this, _)| this.index() == stream_idx) {
			decoder.send_packet(&packet)?;

			let mut decoded = ffmpeg::frame::Audio::empty();
			while decoder.receive_frame(&mut decoded).is_ok() {
				process_frame(&decoded, &mut buffer)?;
			}
		}

		decoder.send_eof()?;

		let mut decoded = ffmpeg::frame::Audio::empty();
		while decoder.receive_frame(&mut decoded).is_ok() {
			process_frame(&decoded, &mut buffer)?;
		}

		{
			let mut f = BufWriter::new(File::create("output.raw")?);
			for s in &buffer {
				f.write_all(&s.to_le_bytes())?;
			}
			f.flush()?;
		}

		buffer
			.chunks(resample_rate)
			.map(|chunk| {
				let sample = chunk.iter().copied().map(|sample| sample.unsigned_abs()).max().unwrap() as f64;

				if self.db_scale {
					util::map2range(
						if sample == 0.0 { f64::NEG_INFINITY } else { 20.0 * sample.log10() },
						self.db_min,
						self.db_max,
						0.0,
						1.0,
					)
				} else {
					util::map2range(sample, 0.0, i16::MIN.unsigned_abs() as f64, 0.0, 1.0)
				}
			})
			.enumerate()
			.try_for_each(|(i, sample)| {
				if i != 0 {
					output.write_all(b",")?;
				}

				write!(output, "{sample:.precision$}", precision = self.precision)?;

				Ok::<_, Error>(())
			})?;

		write!(output, "],\n  \"duration\":{input_duration}\n}}")?;

		Ok(())
	}

	fn open_output_file(&self) -> Result<impl Write, Error> {
		Ok(BufWriter::new(File::create(self.output.as_deref().map(Cow::Borrowed).unwrap_or_else(
			|| {
				let mut file_name = self.input.file_name().unwrap_or(OsStr::new("output")).to_os_string();

				file_name.push(OsStr::new(".json"));

				Cow::Owned(self.input.with_file_name(file_name))
			},
		))?))
	}
}
