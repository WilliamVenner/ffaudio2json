use crate::{
	audio::{PlanarSample, PlanarSampleIteratorEx},
	util::unwrap_break,
	Error,
};
use std::{marker::PhantomData, ops::ControlFlow};

pub struct SampleBuffer<Scalar: PlanarSample, Composite: PlanarSample = Scalar> {
	buffer: Vec<Composite>,
	_phantom: PhantomData<Scalar>,
}
impl<Scalar: PlanarSample, Composite: PlanarSample> SampleBuffer<Scalar, Composite> {
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

			unwrap_break!(process(
				self.buffer
					.drain(..)
					.chain(samples.by_ref().take(samples_take))
					.flatten_samples::<Scalar>()
					.unwrap(),
			)?);
		}

		self.buffer.extend(samples);

		Ok(ControlFlow::Continue(()))
	}

	pub fn flush(&mut self) -> Option<f64> {
		self.buffer.drain(..).flatten_samples::<Scalar>()
	}
}
impl<Scalar: PlanarSample, Composite: PlanarSample> Clone for SampleBuffer<Scalar, Composite> {
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
