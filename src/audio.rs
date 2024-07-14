pub trait PlanarSample: Clone + Copy + Sized + std::cmp::PartialOrd {
	const MAX: Self;
	const MIN: Self;

	/// Normalize the sample to a range of 0.0 to 1.0
	///
	/// We always use `f64` as an intermediary value, this makes it easier to work with composite channels.
	fn normalize(this: f64) -> f64;

	/// Convert the sample to a `f64`
	fn into_f64(self) -> f64;
}

pub trait PackedSample: ffmpeg::frame::audio::Sample {
	/// The planar sample type this sample packs
	type Planar: PlanarSample;

	/// Index the packed sample at the given index, returning the planar sample for the respective channel
	fn index(&self, channel: usize) -> Self::Planar;
}

pub trait PlanarSampleIteratorEx<T: PlanarSample> {
	/// "Flatten" the samples by normalizing them and returning the maximum value
	fn flatten_samples<N: PlanarSample>(self) -> Option<f64>;
}
impl<T: PlanarSample, I: Iterator<Item = T>> PlanarSampleIteratorEx<T> for I {
	fn flatten_samples<N: PlanarSample>(self) -> Option<f64> {
		self.map(PlanarSample::into_f64)
			.map(N::normalize)
			.reduce(|a, b| if a > b { a } else { b })
	}
}

macro_rules! impl_planar_sample {
	(
		signed: $($signed:ty),*;
		signedf: $($signedf:ty),*;
		unsigned: $($unsigned:ty),*;
	) => {
		$(impl PlanarSample for $signed {
			const MAX: Self = Self::MAX;
			const MIN: Self = Self::MIN;

			#[inline]
			fn normalize(this: f64) -> f64 {
				(this.abs() / Self::MAX as f64).min(1.0)
			}

			#[inline]
			fn into_f64(self) -> f64 {
				self as f64
			}
		})*

		$(impl PlanarSample for $signedf {
			const MAX: Self = Self::MAX;
			const MIN: Self = Self::MIN;

			#[inline]
			fn normalize(this: f64) -> f64 {
				this.abs().min(1.0)
			}

			#[inline]
			fn into_f64(self) -> f64 {
				self as f64
			}
		})*

		$(impl PlanarSample for $unsigned {
			const MAX: Self = Self::MAX;
			const MIN: Self = Self::MIN;

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
impl_planar_sample!(
	signed: i32, i16, i64;
	signedf: f32, f64;
	unsigned: u8;
);

macro_rules! impl_packed_sample {
	($($ty:ty: { $($tuple:ty => ($($idx:tt),*),)* },)*) => {
		$($(impl PackedSample for $tuple {
			type Planar = $ty;

			#[inline(always)]
			fn index(&self, channel: usize) -> Self::Planar {
				match channel {
					$($idx => self.$idx,)*
					_ => unreachable!("invalid channel index for packed sample: {channel}"),
				}
			}
		})*)*
	};

	($($ty:ty),*) => {
		impl_packed_sample! {
			$(
				$ty: {
					($ty, $ty) => (0, 1),
					($ty, $ty, $ty) => (0, 1, 2),
					($ty, $ty, $ty, $ty) => (0, 1, 2, 3),
					($ty, $ty, $ty, $ty, $ty) => (0, 1, 2, 3, 4),
					($ty, $ty, $ty, $ty, $ty, $ty) => (0, 1, 2, 3, 4, 5),
					($ty, $ty, $ty, $ty, $ty, $ty, $ty) => (0, 1, 2, 3, 4, 5, 6),
					($ty, $ty, $ty, $ty, $ty, $ty, $ty, $ty) => (0, 1, 2, 3, 4, 5, 6, 7),
				},
			)*
		}
	};
}
impl_packed_sample! { i16, i32, f32, f64, u8 }
