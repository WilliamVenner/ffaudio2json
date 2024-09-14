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

	pub fn push(&mut self, sample: Composite, mut process: impl FnMut(f64) -> Result<ControlFlow<()>, Error>) -> Result<ControlFlow<()>, Error> {
		if self.buffer.len() == self.buffer.capacity() {
			unwrap_break!(process(self.buffer.drain(..).flatten_samples::<Scalar>().unwrap())?);
		}

		self.buffer.push(sample);

		Ok(ControlFlow::Continue(()))
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
		let res = self.buffer.drain(..).flatten_samples::<Scalar>();
		debug_assert!(self.buffer.drain(..).flatten_samples::<Scalar>().is_none());
		res
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

#[test]
fn test_sample_buffer_push() {
	let mut buffer = SampleBuffer::<f64>::with_capacity(10);
	for _ in 0..2 {
		for _ in 0..10 {
			buffer.push(0.5, |_sample| unreachable!()).unwrap();
		}

		buffer
			.push(0.6, |sample| {
				assert_eq!(sample, 0.5);
				Ok(ControlFlow::Continue(()))
			})
			.unwrap();

		assert_eq!(buffer.flush().unwrap(), 0.6);
	}
	assert_eq!(buffer.buffer.capacity(), 10);
}

#[test]
fn test_sample_buffer_extend() {
	let mut buffer = SampleBuffer::<f64>::with_capacity(10);
	for _ in 0..2 {
		buffer.extend((0..10).into_iter().map(|_| 0.5), |_sample| unreachable!()).unwrap();

		buffer
			.extend(std::iter::once(0.6), |sample| {
				assert_eq!(sample, 0.5);
				Ok(ControlFlow::Continue(()))
			})
			.unwrap();

		assert_eq!(buffer.flush().unwrap(), 0.6);
	}
	assert_eq!(buffer.buffer.capacity(), 10);
}

#[test]
fn test_sample_buffer_flush() {
	let mut buffer = SampleBuffer::<f64>::with_capacity(10);
	assert!(buffer.flush().is_none());
	buffer.push(0.5, |_sample| unreachable!()).unwrap();
	assert_eq!(buffer.flush(), Some(0.5));
	assert!(buffer.flush().is_none());
	assert_eq!(buffer.buffer.capacity(), 10);
}
