#[derive(thiserror::Error, Debug)]
/// Errors that can occur when working with this crate
pub enum Error {
	#[error("FFmpeg error: {0}")]
	/// FFmpeg error
	Ffmpeg(#[from] ffmpeg_next::Error),

	#[error("I/O error: {0}")]
	/// I/O error
	Io(#[from] std::io::Error),

	#[error("Format not supported: {format:?}, {channels:?} channels")]
	/// Unsupported format
	UnsupportedFormat {
		/// The format
		format: ffmpeg_next::format::Sample,

		/// The number of channels
		channels: u16,
	},
}
