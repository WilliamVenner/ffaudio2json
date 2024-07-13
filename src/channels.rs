#[derive(Debug, Clone, Copy, strum_macros::EnumString, strum_macros::Display, strum_macros::EnumCount)]
#[strum(serialize_all = "lowercase")]
/// The channels to compute
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
