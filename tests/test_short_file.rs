use ffaudio2json::Channel;

#[macro_use]
mod common;

#[test]
fn test_short_mono_wav_file_trim() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_mono.wav"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_mono_trim.wav.json")))
		.samples(100)
		.channels(vec![Channel::Mid])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_mono_trim.wav.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("mid").unwrap().as_array().unwrap().len(), 100);
}

#[test]
fn test_short_mono_wav_file_full() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_mono.wav"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_mono_full.wav.json")))
		.samples(22932)
		.channels(vec![Channel::Mid])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_mono_full.wav.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("mid").unwrap().as_array().unwrap().len(), 22932);
}

#[test]
fn test_short_mono_wav_file_extended() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_mono.wav"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_mono_extended.wav.json")))
		.samples(50000)
		.channels(vec![Channel::Mid])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_mono_extended.wav.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("mid").unwrap().as_array().unwrap().len(), 22932);

	assert_json_spaces!("airboat_gun_lastshot1_1khz_mono_extended.wav.json", 4);
}

#[test]
fn test_short_mono_flac_file_trim() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_mono.flac"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_mono_trim.flac.json")))
		.samples(100)
		.channels(vec![Channel::Mid])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_mono_trim.flac.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("mid").unwrap().as_array().unwrap().len(), 100);
}

#[test]
fn test_short_mono_flac_file_full() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_mono.flac"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_mono_full.flac.json")))
		.samples(22932)
		.channels(vec![Channel::Mid])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_mono_full.flac.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("mid").unwrap().as_array().unwrap().len(), 22932);
}

#[test]
fn test_short_mono_flac_file_extended() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_mono.flac"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_mono_extended.flac.json")))
		.samples(50000)
		.channels(vec![Channel::Mid])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_mono_extended.flac.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("mid").unwrap().as_array().unwrap().len(), 22932);

	assert_json_spaces!("airboat_gun_lastshot1_1khz_mono_extended.flac.json", 4);
}

#[test]
fn test_short_stereo_wav_file_trim() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_stereo.wav"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_stereo_trim.wav.json")))
		.samples(100)
		.channels(vec![Channel::Left, Channel::Right])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_stereo_trim.wav.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("left").unwrap().as_array().unwrap().len(), 100);
	assert_eq!(json.get("right").unwrap().as_array().unwrap().len(), 100);
}

#[test]
fn test_short_stereo_wav_file_full() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_stereo.wav"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_stereo_full.wav.json")))
		.samples(22932)
		.channels(vec![Channel::Left, Channel::Right])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_stereo_full.wav.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("left").unwrap().as_array().unwrap().len(), 22932);
	assert_eq!(json.get("right").unwrap().as_array().unwrap().len(), 22932);
}

#[test]
fn test_short_stereo_wav_file_extended() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_stereo.wav"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_stereo_extended.wav.json")))
		.samples(50000)
		.channels(vec![Channel::Left, Channel::Right])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_stereo_extended.wav.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("left").unwrap().as_array().unwrap().len(), 22932);
	assert_eq!(json.get("right").unwrap().as_array().unwrap().len(), 22932);

	assert_json_spaces!("airboat_gun_lastshot1_1khz_stereo_extended.wav.json", 6);
}

#[test]
fn test_short_stereo_flac_file_trim() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_stereo.flac"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_stereo_trim.flac.json")))
		.samples(100)
		.channels(vec![Channel::Left, Channel::Right])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_stereo_trim.flac.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("left").unwrap().as_array().unwrap().len(), 100);
	assert_eq!(json.get("right").unwrap().as_array().unwrap().len(), 100);
}

#[test]
fn test_short_stereo_flac_file_full() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_stereo.flac"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_stereo_full.flac.json")))
		.samples(22932)
		.channels(vec![Channel::Left, Channel::Right])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_stereo_full.flac.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("left").unwrap().as_array().unwrap().len(), 22932);
	assert_eq!(json.get("right").unwrap().as_array().unwrap().len(), 22932);
}

#[test]
fn test_short_stereo_flac_file_extended() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("airboat_gun_lastshot1_1khz_stereo.flac"))
		.output(Some(path!("airboat_gun_lastshot1_1khz_stereo_extended.flac.json")))
		.samples(50000)
		.channels(vec![Channel::Left, Channel::Right])
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("airboat_gun_lastshot1_1khz_stereo_extended.flac.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("left").unwrap().as_array().unwrap().len(), 22932);
	assert_eq!(json.get("right").unwrap().as_array().unwrap().len(), 22932);

	assert_json_spaces!("airboat_gun_lastshot1_1khz_stereo_extended.flac.json", 6);
}
