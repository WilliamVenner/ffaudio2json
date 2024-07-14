use ffaudio2json::Channel;
use std::{fs::File, io::BufReader};
use strum::VariantArray;

#[macro_use]
mod common;

#[test]
fn test_stereo_mp3() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("SecondSummerYliStereo.mp3"))
		.output(Some(path!("SecondSummerYliStereo.mp3.json")))
		.samples(800)
		.channels(Channel::VARIANTS.to_vec())
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("SecondSummerYliStereo.mp3.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("left").expect("left missing").as_array().unwrap().len(), 800);
	assert_eq!(json.get("right").expect("right missing").as_array().unwrap().len(), 800);
	assert_eq!(json.get("mid").expect("mid missing").as_array().unwrap().len(), 800);
	assert_eq!(json.get("side").expect("side missing").as_array().unwrap().len(), 800);
	assert_eq!(json.get("min").expect("min missing").as_array().unwrap().len(), 800);
	assert_eq!(json.get("max").expect("max missing").as_array().unwrap().len(), 800);
}

#[test]
fn test_stereo_wav() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("SecondSummerYliStereo.wav"))
		.output(Some(path!("SecondSummerYliStereo.wav.json")))
		.samples(800)
		.channels(Channel::VARIANTS.to_vec())
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("SecondSummerYliStereo.wav.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("left").expect("left missing").as_array().unwrap().len(), 800);
	assert_eq!(json.get("right").expect("right missing").as_array().unwrap().len(), 800);
	assert_eq!(json.get("mid").expect("mid missing").as_array().unwrap().len(), 800);
	assert_eq!(json.get("side").expect("side missing").as_array().unwrap().len(), 800);
	assert_eq!(json.get("min").expect("min missing").as_array().unwrap().len(), 800);
	assert_eq!(json.get("max").expect("max missing").as_array().unwrap().len(), 800);
}
