use crate::{
	audio::{PackedSample, PlanarSample},
	buffer::SampleBuffer,
	channels::{ChannelWriter, Channels},
	util::unwrap_break,
	Error, FfAudio2Json,
};
use std::ops::ControlFlow;

type ChannelBuffers<Planar> = Channels<SampleBuffer<Planar>, SampleBuffer<Planar, f64>>;

macro_rules! flush_channel_buffers {
	($ctx:ident, $channel_buffers:ident, [$($channel:ident),*]) => {
		$(if let Some(ref mut channel) = $channel_buffers.$channel {
			if let Some(sample) = channel.flush() {
				$ctx.writers.$channel.as_mut().unwrap().write(sample, $ctx.config)?;
			}
		})*
	};
}

pub(crate) struct GeneratorContext<'a> {
	pub config: &'a FfAudio2Json,
	pub buffer_capacity: usize,
	pub stream_idx: usize,
	pub writers: Channels<ChannelWriter>,
}
impl<'a> GeneratorContext<'a> {
	pub fn generate(mut self, ictx: &mut ffmpeg::format::context::Input, decoder: &mut ffmpeg::codec::decoder::Audio) -> Result<(), Error> {
		// If there aren't any audio channels, bail
		if decoder.channels() == 0 {
			return Ok(());
		}

		macro_rules! decode {
			($($sample:ident($ty:ty),)*) => {
				match (decoder.format(), decoder.channels()) {
					// Anything with 1 channel can be treated as planar.
					$((ffmpeg::format::Sample::$sample(_), 1) => {
						self.decode::<$ty>(
							|ctx, frame| ctx.decode_planar_frame(frame),
							ictx,
							decoder
						)
					})*

					$((ffmpeg::format::Sample::$sample(ffmpeg::format::sample::Type::Planar), _) => {
						self.decode::<$ty>(
							|ctx, frame| ctx.decode_planar_frame(frame),
							ictx,
							decoder
						)
					})*

					_ => decode!(
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

			(
				$($sample:ident($ty:ty): {
					$($channels:literal => $tuple:ty => ($($idx:tt),*),)*
				},)*
			) => {{
				match (decoder.format(), decoder.channels()) {
					$($((ffmpeg::format::Sample::$sample(ffmpeg::format::sample::Type::Packed), $channels) => self.decode::<$ty>(
						|ctx, frame| ctx.decode_packed_frame::<$tuple>(frame),
						ictx,
						decoder,
					),)*)*

					(format, channels) => Err(Error::UnsupportedFormat { format, channels }),
				}
			}};
		}
		decode! {
			F32(f32),
			I16(i16),
			I32(i32),
			F64(f64),
			U8(u8),
		}
	}

	fn decode<Planar: PlanarSample>(
		&mut self,
		mut frame_decoder: impl for<'frame> FnMut(DecodingContext<Planar>, &'frame ffmpeg::frame::Audio) -> Result<ControlFlow<()>, Error>,
		ictx: &mut ffmpeg::format::context::Input,
		decoder: &mut ffmpeg::codec::decoder::Audio,
	) -> Result<(), Error> {
		let channel_count = decoder.channels() as usize;
		let mut channel_buffers = self.writers.make_buffers::<Planar>(self.buffer_capacity);
		let stream_idx = self.stream_idx;

		macro_rules! frame_decoder {
			($frame:expr) => {
				frame_decoder(
					DecodingContext {
						config: self.config,
						writers: &mut self.writers,
						channel_buffers: &mut channel_buffers,
						channel_count,
					},
					$frame,
				)
			};
		}

		macro_rules! assert_uniform_written {
			() => {
				// Assert all the writers have written the same amount of samples.
				if cfg!(debug_assertions) {
					let written = self.writers.iter_mut().next().unwrap().written;
					self.writers.iter_mut().for_each(|writer| assert_eq!(writer.written, written));
				}
			};
		}

		'overrun: {
			for (_, packet) in ictx.packets().filter(|(this, _)| this.index() == stream_idx) {
				decoder.send_packet(&packet)?;

				let mut decoded = ffmpeg::frame::Audio::empty();
				while decoder.receive_frame(&mut decoded).is_ok() {
					if frame_decoder!(&decoded)?.is_break() {
						break 'overrun;
					}
				}
				assert_uniform_written!();
			}

			decoder.send_eof()?;

			let mut decoded = ffmpeg::frame::Audio::empty();
			while decoder.receive_frame(&mut decoded).is_ok() {
				if frame_decoder!(&decoded)?.is_break() {
					break 'overrun;
				}
			}
			assert_uniform_written!();

			if channel_count == 1 {
				// If there's only 1 channel, we need to flush the first buffer, and write it to all the channel writers.

				// Weird iterator drop glue stuff requires this weird looking code.
				let scalar = { channel_buffers.iter_mut_scalar().next().map(|buffer| buffer.flush()) };

				let composite = if scalar.is_none() {
					channel_buffers.iter_mut_composite().next().map(|buffer| buffer.flush())
				} else {
					None
				};

				if let Some(sample) = scalar.or(composite).flatten() {
					for writer in self.writers.iter_mut() {
						writer.write(sample, self.config)?;
					}
				}
			} else {
				flush_channel_buffers!(self, channel_buffers, [left, right, mid, side, min, max]);
			}
		}

		debug_assert!(
			channel_buffers.iter_mut_scalar().all(|buffer| buffer.flush().is_none()),
			"Scalar buffers not flushed (channel_count={channel_count})"
		);
		debug_assert!(
			channel_buffers.iter_mut_composite().all(|buffer| buffer.flush().is_none()),
			"Composite buffers not flushed (channel_count={channel_count})"
		);

		Ok(())
	}
}

struct DecodingContext<'a, 'b, 'c, Planar: PlanarSample> {
	config: &'a FfAudio2Json,
	writers: &'b mut Channels<ChannelWriter>,
	channel_buffers: &'c mut ChannelBuffers<Planar>,
	channel_count: usize,
}
impl<Planar: PlanarSample> DecodingContext<'_, '_, '_, Planar> {
	fn decode_planar_frame(self, decoded: &ffmpeg::frame::Audio) -> Result<ControlFlow<()>, Error>
	where
		Planar: ffmpeg::frame::audio::Sample,
	{
		macro_rules! push_to_writer {
			($sample:expr => $channel:ident) => {
				if let Some(ref mut writer) = self.writers.$channel {
					unwrap_break!(
						self.channel_buffers.$channel.as_mut().unwrap().push(
							$sample,
							|sample| writer.write(sample, self.config).map_err(Into::into)
						)?
					);
				}
			};

			($channel:ident) => {
				push_to_writer!($channel => $channel);
			};
		}

		if self.channel_count == 1 {
			// If there's only 1 channel, we can just dump it as-is to all the channel writers.

			macro_rules! dump_to_writer {
				($buffer:ident => $extend:expr) => {
					return $buffer.extend($extend, |sample| {
						for writer in self.writers.iter_mut() {
							if writer.write(sample, self.config)?.is_break() {
								return Ok(ControlFlow::Break(()));
							}
						}
						Ok(ControlFlow::Continue(()))
					});
				};
			}

			if let Some(buffer) = self.channel_buffers.iter_mut_scalar().next() {
				dump_to_writer!(buffer => decoded.plane::<Planar>(0).iter().copied());
			}

			if let Some(buffer) = self.channel_buffers.iter_mut_composite().next() {
				dump_to_writer!(buffer => decoded.plane::<Planar>(0).iter().copied().map(|sample| sample.into_f64()));
			}

		// BTW, we also need to do this after flushing the first buffer, as it's the only buffer we pushed to.
		} else {
			for sample in 0..decoded.samples() {
				let mut min = Planar::MAX;
				let mut max = Planar::MIN;
				let mut sum = 0.0_f64;

				for (plane, sample) in (0..decoded.planes())
					.map(|plane| (plane, decoded.plane::<Planar>(plane)))
					.map(|(i, plane)| (i, plane[sample]))
				{
					match plane {
						0 => push_to_writer!(sample => left),
						1 => push_to_writer!(sample => right),
						_ => {}
					}

					sum += sample.into_f64();

					if sample < min {
						min = sample;
					}

					if sample > max {
						max = sample;
					}
				}

				push_to_writer!(min);
				push_to_writer!(max);

				let mid = sum / decoded.planes() as f64;
				push_to_writer!(mid);

				let left = decoded.plane::<Planar>(0)[sample];
				let right = decoded.plane::<Planar>(1)[sample];
				let side = left.into_f64() - right.into_f64();
				push_to_writer!(side);
			}
		}

		Ok::<_, Error>(ControlFlow::Continue(()))
	}

	fn decode_packed_frame<Packed: PackedSample<Planar = Planar>>(self, decoded: &ffmpeg::frame::Audio) -> Result<ControlFlow<()>, Error> {
		debug_assert_eq!(decoded.planes(), 1);

		let plane = decoded.plane::<Packed>(0);

		debug_assert_eq!(plane.len() % self.channel_count, 0);

		if self.channel_count == 1 {
			for sample in plane {
				macro_rules! push_to_writer {
					($idx:literal => @composite $writer:ident) => {
						if let Some(ref mut writer) = self.writers.$writer {
							unwrap_break!(self
								.channel_buffers
								.$writer
								.as_mut()
								.unwrap()
								.push(sample.index($idx).into_f64(), |sample| writer
									.write(sample, self.config)
									.map_err(Into::into))?);
						}
					};

					($idx:literal => $writer:ident) => {
						if let Some(ref mut writer) = self.writers.$writer {
							unwrap_break!(self
								.channel_buffers
								.$writer
								.as_mut()
								.unwrap()
								.push(sample.index($idx), |sample| {
									writer.write(sample, self.config).map_err(Into::into)
								})?);
						}
					};
				}

				// If there's only 1 channel, we can just dump it as-is to all the channel writers.
				push_to_writer!(0 => left);
				push_to_writer!(0 => right);
				push_to_writer!(0 => min);
				push_to_writer!(0 => max);
				push_to_writer!(0 => @composite mid);
				push_to_writer!(0 => @composite side);
			}
		} else {
			macro_rules! impl_channels {
				(
					let $sample:ident;
					$($channel:ident => || $transform:expr;)*
				) => {
					for $sample in plane {
						$(if let Some(ref mut channel) = self.writers.$channel {
							unwrap_break!(
								self.channel_buffers.$channel.as_mut().unwrap().push(
									$transform,
									|sample| channel.write(sample, self.config).map_err(Into::into)
								)?
							);
						})*
					}
				};
			}
			impl_channels!(
				let sample;

				left => || sample.index(0);

				right => || sample.index(1);

				mid => || (0..self.channel_count).map(|channel| sample.index(channel).into_f64()).sum::<f64>() / self.channel_count as f64;

				side => || {
					let left = sample.index(0).into_f64();
					let right = sample.index(1).into_f64();
					(left - right) / 2.0
				};

				min => || {
					(0..self.channel_count)
					.map(|channel| sample.index(channel))
					.reduce(|a, b| if a < b { a } else { b })
					.unwrap()
				};

				max => || {
					(0..self.channel_count)
					.map(|channel| sample.index(channel))
					.reduce(|a, b| if a > b { a } else { b })
					.unwrap()
				};
			);
		}

		Ok::<_, Error>(ControlFlow::Continue(()))
	}
}
