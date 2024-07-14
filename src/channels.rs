use crate::{audio::PlanarSample, buffer::SampleBuffer, util, FfAudio2Json};
use std::{
	fs::File,
	io::{BufWriter, Write},
	ops::ControlFlow,
};

#[derive(
	Debug, Clone, Copy, strum_macros::EnumString, strum_macros::Display, strum_macros::EnumCount, strum_macros::EnumIter, strum_macros::VariantArray,
)]
#[strum(serialize_all = "lowercase")]
/// Channels that FFAudio2JSON can output
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

pub(crate) struct ChannelWriter {
	inner: BufWriter<File>,
	written: usize,
}
impl ChannelWriter {
	pub(crate) fn new(writer: BufWriter<File>) -> Self {
		Self { inner: writer, written: 0 }
	}

	pub(crate) fn write(&mut self, mut sample: f64, config: &FfAudio2Json) -> Result<ControlFlow<()>, std::io::Error> {
		debug_assert!(sample >= 0.0);

		self.written += 1;

		if self.written > config.samples as usize {
			return Ok(ControlFlow::Break(()));
		}

		if config.db_scale {
			sample = util::map2range(
				if sample > 0.0 { 20.0 * sample.log10() } else { config.db_min },
				config.db_min,
				config.db_max,
				0.0,
				1.0,
			);
		}

		if self.written != 1 {
			write!(self.inner, ",")?;
		}

		write!(self.inner, "{sample:.precision$}", precision = config.precision)?;

		Ok(ControlFlow::Continue(()))
	}
}

pub(crate) struct Channels<Scalar, Composite = Scalar> {
	pub(crate) left: Option<Scalar>,
	pub(crate) right: Option<Scalar>,
	pub(crate) mid: Option<Composite>,
	pub(crate) side: Option<Composite>,
	pub(crate) min: Option<Scalar>,
	pub(crate) max: Option<Scalar>,
}
impl<Scalar, Composite> Default for Channels<Scalar, Composite> {
	fn default() -> Self {
		Self {
			left: None,
			right: None,
			mid: None,
			side: None,
			min: None,
			max: None,
		}
	}
}
impl<Scalar, Composite> Channels<Scalar, Composite> {
	pub fn iter_mut_scalar(&mut self) -> impl Iterator<Item = &mut Scalar> {
		[self.left.as_mut(), self.right.as_mut(), self.min.as_mut(), self.max.as_mut()]
			.into_iter()
			.flatten()
	}

	pub fn iter_mut_composite(&mut self) -> impl Iterator<Item = &mut Composite> {
		[self.mid.as_mut(), self.side.as_mut()].into_iter().flatten()
	}
}
impl<Scalar> Channels<Scalar, Scalar> {
	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Scalar> {
		[
			self.left.as_mut(),
			self.right.as_mut(),
			self.mid.as_mut(),
			self.side.as_mut(),
			self.min.as_mut(),
			self.max.as_mut(),
		]
		.into_iter()
		.flatten()
	}
}
impl Channels<ChannelWriter> {
	pub(crate) fn make_buffers<Scalar: PlanarSample>(&self, capacity: usize) -> Channels<SampleBuffer<Scalar>, SampleBuffer<Scalar, f64>> {
		Channels {
			left: self.left.as_ref().map(|_| SampleBuffer::with_capacity(capacity)),
			right: self.right.as_ref().map(|_| SampleBuffer::with_capacity(capacity)),
			mid: self.mid.as_ref().map(|_| SampleBuffer::with_capacity(capacity)),
			side: self.side.as_ref().map(|_| SampleBuffer::with_capacity(capacity)),
			min: self.min.as_ref().map(|_| SampleBuffer::with_capacity(capacity)),
			max: self.max.as_ref().map(|_| SampleBuffer::with_capacity(capacity)),
		}
	}
}
