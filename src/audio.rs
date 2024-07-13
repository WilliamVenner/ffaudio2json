use crate::{Error, GeneratorContext};
use std::{
	marker::PhantomData,
	ops::{Add, ControlFlow, Div, Sub},
};

macro_rules! flush_channel_buffers {
	($ctx:ident, $channel_buffers:ident, [$($channel:ident),*]) => {
		'overrun: {
			$(if let Some(ref mut channel) = $channel_buffers.$channel {
				if let Some(sample) = channel.flush() {
					if $ctx.channels.$channel.as_mut().unwrap().write(sample, $ctx.config)?.is_break() {
						break 'overrun;
					}
				}
			})*
		}
	};
}

pub struct AudioBuffer<Scalar: PlanarAudioSample, Composite: PlanarAudioSample = Scalar> {
	buffer: Vec<Composite>,
	_phantom: PhantomData<Scalar>,
}
impl<Scalar: PlanarAudioSample, Composite: PlanarAudioSample> AudioBuffer<Scalar, Composite> {
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			buffer: Vec::with_capacity(capacity),
			_phantom: PhantomData,
		}
	}

	pub fn push(&mut self, sample: Composite) -> Option<f64> {
		if self.buffer.len() != self.buffer.capacity() {
			self.buffer.push(sample);
			None
		} else {
			self.buffer.drain(..).chain(std::iter::once(sample)).flatten_samples::<Scalar>()
		}
	}

	pub fn extend(
		&mut self,
		mut samples: impl ExactSizeIterator<Item = Composite>,
		mut process: impl FnMut(f64) -> Result<ControlFlow<()>, Error>,
	) -> Result<ControlFlow<()>, Error> {
		while self.buffer.len() + samples.len() > self.buffer.capacity() {
			let samples_take = self.buffer.capacity() - self.buffer.len();

			if process(
				self.buffer
					.drain(..)
					.chain(samples.by_ref().take(samples_take))
					.flatten_samples::<Scalar>()
					.unwrap(),
			)?
			.is_break()
			{
				return Ok(ControlFlow::Break(()));
			}
		}

		self.buffer.extend(samples);

		Ok(ControlFlow::Continue(()))
	}

	pub fn flush(&mut self) -> Option<f64> {
		self.buffer.drain(..).flatten_samples::<Scalar>()
	}
}
impl<Scalar: PlanarAudioSample, Composite: PlanarAudioSample> Clone for AudioBuffer<Scalar, Composite> {
	fn clone(&self) -> Self {
		Self {
			buffer: {
				let mut buffer = Vec::with_capacity(self.buffer.capacity());
				buffer.extend(self.buffer.iter().copied());
				buffer
			},
			_phantom: PhantomData,
		}
	}
}

pub fn process(
	ictx: &mut ffmpeg::format::context::Input,
	decoder: &mut ffmpeg::codec::decoder::Audio,
	stream_idx: usize,
	mut ctx: GeneratorContext,
) -> Result<(), Error> {
	macro_rules! process_packed {
		($($sample:ident($ty:ty): {
			$($channels:literal => $tuple:ty => ($($idx:tt),*),)*
		},)*) => {{
			$($(impl PackedAudioSample for $tuple {
				type PlanarAudioSample = $ty;

				#[inline(always)]
				fn index_tuple(&self, index: usize) -> Self::PlanarAudioSample {
					match index {
						$($idx => self.$idx,)*
						_ => unreachable!(),
					}
				}
			})*)*

			match (decoder.format(), decoder.channels()) {
				$($((ffmpeg::format::Sample::$sample(ffmpeg::format::sample::Type::Packed), $channels) => run_packed::<$ty, $tuple>(
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
				$((ffmpeg::format::Sample::$sample(_), 1) => {
					run_planar::<$ty>(ictx, decoder, stream_idx, &mut ctx)
				})*

				$((ffmpeg::format::Sample::$sample(ffmpeg::format::sample::Type::Planar), _) => {
					run_planar::<$ty>(ictx, decoder, stream_idx, &mut ctx)
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
	ictx: &mut ffmpeg::format::context::Input,
	decoder: &mut ffmpeg::codec::decoder::Audio,
	stream_idx: usize,
	ctx: &mut GeneratorContext,
) -> Result<(), Error> {
	let mut channel_buffers = ctx.channels.make_buffers::<T>(ctx.buffer_capacity);

	let mut process_frame = |decoded: &ffmpeg::frame::Audio| {
		macro_rules! dump_to_writer {
			($idx:literal => $writer:ident $(=> |$ident:ident| $map:expr)?) => {
				if let Some(ref mut writer) = ctx.channels.$writer {
					if channel_buffers
						.$writer
						.as_mut()
						.unwrap()
						.extend(
							decoded.plane::<T>($idx).iter().copied().map(|sample| {
								$(let sample = {
									let $ident = sample;
									$map
								};)?
								sample
							}),
							|sample| writer.write(sample, ctx.config).map_err(Into::into),
						)?
						.is_break()
					{
						return Ok(ControlFlow::Break(()));
					}
				}
			};
		}

		match decoded.planes() {
			0 => {}

			1 => {
				dump_to_writer!(0 => left);
				dump_to_writer!(0 => mid => |sample| sample.into_f64());
			}

			_ => {
				dump_to_writer!(0 => left);
				dump_to_writer!(1 => right);

				if let Some(ref mut mid) = ctx.channels.mid {
					if channel_buffers
						.mid
						.as_mut()
						.unwrap()
						.extend(
							(0..decoded.samples()).map(|sample| {
								(0..decoded.planes())
									.map(|plane| decoded.plane::<T>(plane))
									.map(|plane| plane[sample])
									.map(|sample| sample.into_f64())
									.sum::<f64>() / decoded.planes() as f64
							}),
							|sample| mid.write(sample, ctx.config).map_err(Into::into),
						)?
						.is_break()
					{
						return Ok(ControlFlow::Break(()));
					}
				}

				if let Some(ref mut side) = ctx.channels.side {
					if channel_buffers
						.side
						.as_mut()
						.unwrap()
						.extend(
							decoded
								.plane::<T>(0)
								.iter()
								.copied()
								.zip(decoded.plane::<T>(1).iter().copied())
								.map(|(left, right)| (left.into_f64(), right.into_f64()))
								.map(|(left, right)| (left - right) / 2.0),
							|sample| side.write(sample, ctx.config).map_err(Into::into),
						)?
						.is_break()
					{
						return Ok(ControlFlow::Break(()));
					}
				}

				if let Some(ref mut min) = ctx.channels.min {
					if channel_buffers
						.min
						.as_mut()
						.unwrap()
						.extend(
							(0..decoded.samples()).map(|sample| {
								(0..decoded.planes())
									.map(|plane| decoded.plane::<T>(plane))
									.map(|plane| plane[sample])
									.reduce(|a, b| if a < b { a } else { b })
									.unwrap()
							}),
							|sample| min.write(sample, ctx.config).map_err(Into::into),
						)?
						.is_break()
					{
						return Ok(ControlFlow::Break(()));
					}
				}

				if let Some(ref mut max) = ctx.channels.max {
					if channel_buffers
						.max
						.as_mut()
						.unwrap()
						.extend(
							(0..decoded.samples()).map(|sample| {
								(0..decoded.planes())
									.map(|plane| decoded.plane::<T>(plane))
									.map(|plane| plane[sample])
									.reduce(|a, b| if a > b { a } else { b })
									.unwrap()
							}),
							|sample| max.write(sample, ctx.config).map_err(Into::into),
						)?
						.is_break()
					{
						return Ok(ControlFlow::Break(()));
					}
				}
			}
		}

		Ok::<_, Error>(ControlFlow::Continue(()))
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

	flush_channel_buffers!(ctx, channel_buffers, [left, right, mid, side, min, max]);

	Ok(())
}

fn run_packed<T: PlanarAudioSample, F: PackedAudioSample<PlanarAudioSample = T>>(
	ictx: &mut ffmpeg::format::context::Input,
	decoder: &mut ffmpeg::codec::decoder::Audio,
	stream_idx: usize,
	ctx: &mut GeneratorContext,
) -> Result<(), Error> {
	let channel_count = decoder.channels() as usize;

	let mut channel_buffers = ctx.channels.make_buffers::<T>(ctx.buffer_capacity);

	let mut process_frame = |decoded: &ffmpeg::frame::Audio| {
		debug_assert_eq!(decoded.planes(), 1);

		let plane = decoded.plane::<F>(0);

		debug_assert_eq!(plane.len() % channel_count, 0);

		// TODO channel_count 1 optimisation

		for sample in plane {
			if let Some(ref mut left) = ctx.channels.left {
				if let Some(sample) = channel_buffers.left.as_mut().unwrap().push(sample.index_tuple(0)) {
					if left.write(sample, ctx.config)?.is_break() {
						return Ok(ControlFlow::Break(()));
					}
				}
			}

			if let Some(ref mut right) = ctx.channels.right {
				if let Some(sample) = channel_buffers.right.as_mut().unwrap().push(sample.index_tuple(1)) {
					if right.write(sample, ctx.config)?.is_break() {
						return Ok(ControlFlow::Break(()));
					}
				}
			}

			if let Some(ref mut mid) = ctx.channels.mid {
				let sample = if channel_count != 1 {
					(0..channel_count).map(|channel| sample.index_tuple(channel).into_f64()).sum::<f64>() / channel_count as f64
				} else {
					sample.index_tuple(0).into_f64()
				};

				if let Some(sample) = channel_buffers.mid.as_mut().unwrap().push(sample) {
					if mid.write(sample, ctx.config)?.is_break() {
						return Ok(ControlFlow::Break(()));
					}
				}
			}

			if let Some(ref mut side) = ctx.channels.side {
				let sample = (sample.index_tuple(0).into_f64() - sample.index_tuple(1).into_f64()) / 2.0;

				if let Some(sample) = channel_buffers.side.as_mut().unwrap().push(sample) {
					if side.write(sample, ctx.config)?.is_break() {
						return Ok(ControlFlow::Break(()));
					}
				}
			}

			if let Some(ref mut min) = ctx.channels.min {
				let sample = (0..channel_count)
					.map(|channel| sample.index_tuple(channel))
					.reduce(|a, b| if a < b { a } else { b })
					.unwrap();

				if let Some(sample) = channel_buffers.min.as_mut().unwrap().push(sample) {
					if min.write(sample, ctx.config)?.is_break() {
						return Ok(ControlFlow::Break(()));
					}
				}
			}

			if let Some(ref mut max) = ctx.channels.max {
				let sample = (0..channel_count)
					.map(|channel| sample.index_tuple(channel))
					.reduce(|a, b| if a > b { a } else { b })
					.unwrap();

				if let Some(sample) = channel_buffers.max.as_mut().unwrap().push(sample) {
					if max.write(sample, ctx.config)?.is_break() {
						return Ok(ControlFlow::Break(()));
					}
				}
			}
		}

		Ok::<_, Error>(ControlFlow::Continue(()))
	};

	'overrun: {
		for (_, packet) in ictx.packets().filter(|(this, _)| this.index() == stream_idx) {
			decoder.send_packet(&packet)?;

			let mut decoded = ffmpeg::frame::Audio::empty();
			while decoder.receive_frame(&mut decoded).is_ok() {
				if process_frame(&decoded)?.is_break() {
					break 'overrun;
				}
			}
		}

		decoder.send_eof()?;

		let mut decoded = ffmpeg::frame::Audio::empty();
		while decoder.receive_frame(&mut decoded).is_ok() {
			if process_frame(&decoded)?.is_break() {
				break 'overrun;
			}
		}

		flush_channel_buffers!(ctx, channel_buffers, [left, right, mid, side, min, max]);
	}

	Ok(())
}

pub trait PlanarAudioSample:
	Clone + Copy + Sized + Add<Output = Self> + Sub<Output = Self> + Div<Output = Self> + std::iter::Sum<Self> + std::cmp::PartialOrd
{
	fn normalize(this: f64) -> f64;
	fn into_f64(self) -> f64;
}

macro_rules! impl_planar_audio_sample {
	(
		signed: $($signed:ty),*;
		signedf: $($signedf:ty),*;
		unsigned: $($unsigned:ty),*;
	) => {
		$(impl PlanarAudioSample for $signed {
			#[inline]
			fn normalize(this: f64) -> f64 {
				(this.abs() / Self::MAX as f64).min(1.0)
			}

			#[inline]
			fn into_f64(self) -> f64 {
				self as f64
			}
		})*

		$(impl PlanarAudioSample for $signedf {
			#[inline]
			fn normalize(this: f64) -> f64 {
				this.abs().min(1.0)
			}

			#[inline]
			fn into_f64(self) -> f64 {
				self as f64
			}
		})*

		$(impl PlanarAudioSample for $unsigned {
			#[inline]
			fn normalize(this: f64) -> f64 {
				(this / Self::MAX as f64).min(1.0)
			}

			#[inline]
			fn into_f64(self) -> f64 {
				self as f64
			}
		})*
	};
}
impl_planar_audio_sample!(
	signed: i32, i16, i64;
	signedf: f32, f64;
	unsigned: u8;
);

trait PackedAudioSample: ffmpeg::frame::audio::Sample {
	type PlanarAudioSample: PlanarAudioSample;

	fn index_tuple(&self, index: usize) -> Self::PlanarAudioSample;
}

trait PlanarAudioSampleIterator<T: PlanarAudioSample> {
	fn flatten_samples<N: PlanarAudioSample>(self) -> Option<f64>;
}
impl<T: PlanarAudioSample, I: Iterator<Item = T>> PlanarAudioSampleIterator<T> for I {
	fn flatten_samples<N: PlanarAudioSample>(self) -> Option<f64> {
		self.map(PlanarAudioSample::into_f64)
			.map(N::normalize)
			.reduce(|a, b| if a > b { a } else { b })
	}
}
