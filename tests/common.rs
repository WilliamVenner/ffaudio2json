#[macro_export]
macro_rules! path {
	($path:literal) => {
		::std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/", $path))
	};
}

#[macro_export]
macro_rules! open_json {
	($path:literal) => {
		::serde_json::from_reader::<_, serde_json::Value>(::std::io::BufReader::new(::std::fs::File::open(path!($path)).unwrap())).unwrap()
	};
}

#[macro_export]
macro_rules! assert_json_spaces {
	($path:literal, $spaces:literal) => {{
		assert_eq!(::std::fs::read_to_string(path!($path)).unwrap().matches(' ').count(), $spaces)
	}};
}

#[macro_export]
macro_rules! enable_logging {
	() => {
		::stderrlog::new()
			.module("ffaudio2json")
			.verbosity(::log::Level::Debug)
			.timestamp(::stderrlog::Timestamp::Millisecond)
			.init()
			.ok();
	};
}
