use crate::{util, ChannelWriter, Config, Error, GeneratorContext};
use std::{io::Write, marker::PhantomData};

#[derive(Clone)]
pub struct AudioBuffer<T: PlanarAudioSample> {
	buffer: Vec<T>,
	_phantom: PhantomData<T>,
}
impl<T: PlanarAudioSample> AudioBuffer<T> {
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			buffer: Vec::with_capacity(capacity),
			_phantom: PhantomData,
		}
	}

	pub fn push(&mut self, sample: T, config: &Config, writer: &mut ChannelWriter) -> Result<(), Error> {
		if self.buffer.len() != self.buffer.capacity() {
			self.buffer.push(sample);
			Ok(())
		} else {
			Self::write(self.buffer.drain(..).chain(std::iter::once(sample)), config, writer)
		}
	}

	pub fn extend(&mut self, samples: &[T], config: &Config, writer: &mut ChannelWriter) -> Result<(), Error> {
		let mut samples = samples.iter().copied();

		// If we can fit the new samples into the buffer, just extend it
		if self.buffer.len() + samples.len() < self.buffer.capacity() {
			self.buffer.extend(samples);
			return Ok(());
		}

		let samples_take = self.buffer.capacity() - self.buffer.len();

		Self::write(self.buffer.drain(..).chain(samples.by_ref().take(samples_take)), config, writer)?;

		self.buffer.extend(samples);

		Ok(())
	}

	pub fn flush(&mut self, config: &Config, writer: &mut ChannelWriter) -> Result<(), Error> {
		Self::write(self.buffer.drain(..), config, writer)?;
		writer.inner.flush()?;
		Ok(())
	}

	fn write(samples: impl Iterator<Item = T>, config: &Config, writer: &mut ChannelWriter) -> Result<(), Error> {
		let Some(mut sample) = samples.map(PlanarAudioSample::normalize).reduce(|a, b| if a > b { a } else { b }) else {
			return Ok(());
		};

		if config.db_scale {
			sample = util::map2range(
				if sample == 0.0 { f64::NEG_INFINITY } else { 20.0 * sample.log10() },
				config.db_min,
				config.db_max,
				0.0,
				1.0,
			);
		}

		writer.write(sample, config)?;

		Ok(())
	}
}

pub fn process(
	ictx: &mut ffmpeg::format::context::Input,
	decoder: &mut ffmpeg::codec::decoder::Audio,
	stream_idx: usize,
	buffer_capacity: usize,
	mut ctx: GeneratorContext,
) -> Result<(), Error> {
	macro_rules! iter_packed_interleaved {
		($packed:ident, $($idx:tt),*) => {
			[$(($idx, $packed.$idx)),*]
		};
	}

	macro_rules! process_packed {
		($($sample:ident($ty:ty): {
			$($channels:literal => $tuple:ty => ($($idx:tt),*),)*
		},)*) => {{
			$($(impl PackedAudioSamples for [$tuple] {
				type PlanarAudioSample = $ty;

				fn iter_packed_interleaved(&self) -> impl Iterator<Item = (usize, Self::PlanarAudioSample)> {
					self.iter()
						.flat_map(move |packed| iter_packed_interleaved!(packed, $($idx),*))
				}
			})*)*

			match (decoder.format(), decoder.channels()) {
				$($((ffmpeg::format::Sample::$sample(ffmpeg::format::sample::Type::Packed), $channels) => run_packed::<$ty, $tuple>(
					&mut std::array::from_fn::<_, $channels, _>(|_| AudioBuffer::with_capacity(buffer_capacity)),
					ictx,
					decoder,
					stream_idx,
					&mut ctx,
				),)*)*

				(format, channels) => Err(Error::UnsupportedFormat { format, channels }),
			}
		}};
	}

	macro_rules! process_planar_channels {
		($($sample:ident($ty:ty): [$($channels:literal),*],)*) => {
			match (decoder.format(), decoder.channels()) {
				$($((ffmpeg::format::Sample::$sample(_), $channels) => {
					run_planar::<$ty>(&mut std::array::from_fn::<_, $channels, _>(|_| AudioBuffer::with_capacity(buffer_capacity)), ictx, decoder, stream_idx, &mut ctx)
				})*)*

				$((ffmpeg::format::Sample::$sample(_), channels) => {
					run_planar::<$ty>(&mut vec![AudioBuffer::with_capacity(buffer_capacity); channels as usize], ictx, decoder, stream_idx, &mut ctx)
				})*

				_ => process_packed!(
					$(
						$sample($ty): {
							2 => ($ty, $ty) => (0, 1),
							3 => ($ty, $ty, $ty) => (0, 1, 2),
							4 => ($ty, $ty, $ty, $ty) => (0, 1, 2, 3),
							5 => ($ty, $ty, $ty, $ty, $ty) => (0, 1, 2, 3, 4),
							6 => ($ty, $ty, $ty, $ty, $ty, $ty) => (0, 1, 2, 3, 4, 5),
							7 => ($ty, $ty, $ty, $ty, $ty, $ty, $ty) => (0, 1, 2, 3, 4, 5, 6),
							8 => ($ty, $ty, $ty, $ty, $ty, $ty, $ty, $ty) => (0, 1, 2, 3, 4, 5, 6, 7),
						},
					)*
				),
			}
		};
	}

	macro_rules! process {
		($($sample:ident($ty:ty),)*) => {
			process_planar_channels!($($sample($ty): [1,2,3,4,5,6,7,8],)*)
		};
	}
	process! {
		F32(f32),
		I16(i16),
		I32(i32),
		F64(f64),
		U8(u8),
	}
}

fn run_planar<T: PlanarAudioSample + ffmpeg::frame::audio::Sample>(
	channel_buffers: &mut [AudioBuffer<T>],
	ictx: &mut ffmpeg::format::context::Input,
	decoder: &mut ffmpeg::codec::decoder::Audio,
	stream_idx: usize,
	ctx: &mut GeneratorContext,
) -> Result<(), Error> {
	let mut process_frame = |decoded: &ffmpeg::frame::Audio| {
		(0..decoded.planes()).try_for_each(|plane| {
			let channel_buffer = &mut channel_buffers[plane];
			let writer = ctx.channel_writers[plane].as_mut().unwrap();
			let plane = decoded.plane::<T>(plane);
			channel_buffer.extend(plane, ctx.config, writer)
		})
	};

	for (_, packet) in ictx.packets().filter(|(this, _)| this.index() == stream_idx) {
		decoder.send_packet(&packet)?;

		let mut decoded = ffmpeg::frame::Audio::empty();
		while decoder.receive_frame(&mut decoded).is_ok() {
			process_frame(&decoded)?;
		}
	}

	decoder.send_eof()?;

	let mut decoded = ffmpeg::frame::Audio::empty();
	while decoder.receive_frame(&mut decoded).is_ok() {
		process_frame(&decoded)?;
	}

	for (channel_buffer, writer) in channel_buffers
		.iter_mut()
		.zip(ctx.channel_writers.iter_mut().map(|writer| writer.as_mut().unwrap()))
	{
		channel_buffer.flush(ctx.config, writer)?;
	}

	Ok(())
}

fn run_packed<T: PlanarAudioSample, F: ffmpeg::frame::audio::Sample>(
	channel_buffers: &mut [AudioBuffer<T>],
	ictx: &mut ffmpeg::format::context::Input,
	decoder: &mut ffmpeg::codec::decoder::Audio,
	stream_idx: usize,
	ctx: &mut GeneratorContext,
) -> Result<(), Error>
where
	[F]: PackedAudioSamples<PlanarAudioSample = T>,
{
	let mut process_frame = |decoded: &ffmpeg::frame::Audio| {
		debug_assert_eq!(decoded.planes(), 1);

		let plane = decoded.plane::<F>(0);

		plane.iter_packed_interleaved().try_for_each(|(channel_buffer, sample)| {
			let channel_writer = ctx.channel_writers[channel_buffer].as_mut().unwrap();
			let channel_buffer = &mut channel_buffers[channel_buffer];
			channel_buffer.push(sample, ctx.config, channel_writer)
		})
	};

	for (_, packet) in ictx.packets().filter(|(this, _)| this.index() == stream_idx) {
		decoder.send_packet(&packet)?;

		let mut decoded = ffmpeg::frame::Audio::empty();
		while decoder.receive_frame(&mut decoded).is_ok() {
			process_frame(&decoded)?;
		}
	}

	decoder.send_eof()?;

	let mut decoded = ffmpeg::frame::Audio::empty();
	while decoder.receive_frame(&mut decoded).is_ok() {
		process_frame(&decoded)?;
	}

	for (channel_buffer, writer) in channel_buffers
		.iter_mut()
		.zip(ctx.channel_writers.iter_mut().map(|writer| writer.as_mut().unwrap()))
	{
		channel_buffer.flush(ctx.config, writer)?;
	}

	Ok(())
}

pub trait PlanarAudioSample: Clone + Copy + Sized {
	fn normalize(self) -> f64;
}

macro_rules! impl_planar_audio_sample {
	(
		signed: $($signed:ty),*;
		signedf: $($signedf:ty),*;
		unsigned: $($unsigned:ty),*;
	) => {
		$(impl PlanarAudioSample for $signed {
			#[inline]
			fn normalize(self) -> f64 {
				(self.unsigned_abs() as f64 / Self::MAX as f64).min(1.0)
			}
		})*

		$(impl PlanarAudioSample for $signedf {
			#[inline]
			fn normalize(self) -> f64 {
				self.abs() as _
			}
		})*

		$(impl PlanarAudioSample for $unsigned {
			#[inline]
			fn normalize(self) -> f64 {
				self as f64 / Self::MAX as f64
			}
		})*
	};
}
impl_planar_audio_sample!(
	signed: i32, i16, i64;
	signedf: f32, f64;
	unsigned: u8;
);

trait PackedAudioSamples {
	type PlanarAudioSample: PlanarAudioSample;

	fn iter_packed_interleaved(&self) -> impl Iterator<Item = (usize, Self::PlanarAudioSample)>;
}
