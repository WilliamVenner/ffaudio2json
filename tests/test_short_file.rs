use ffaudio2json::Channel;

#[macro_use]
mod common;

#[test]
fn test_short_file_trim() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz.wav"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_trim.wav.json")))
		.samples(100)
		.channels(vec![Channel::Mid])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_trim.wav.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("mid").unwrap().as_array().unwrap().len(), 100);
}

#[test]
fn test_short_file_full() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz.wav"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_full.wav.json")))
		.samples(22932)
		.channels(vec![Channel::Mid])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_full.wav.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("mid").unwrap().as_array().unwrap().len(), 22932);
}

#[test]
fn test_short_file_extended() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz.wav"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_extended.wav.json")))
		.samples(50000)
		.channels(vec![Channel::Mid])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_extended.wav.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("mid").unwrap().as_array().unwrap().len(), 22932);

	let spaces = std::fs::read_to_string(path!("airboat_gun_lastshot1_1khz_extended.wav.json"))
		.unwrap()
		.chars()
		.filter(|char| *char == ' ')
		.count();

	assert_eq!(spaces, 4);
}
