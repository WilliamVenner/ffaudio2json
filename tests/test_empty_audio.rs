use ffaudio2json::Channel;
use strum::VariantArray;

#[macro_use]
mod common;

#[test]
fn test_empty_audio() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("empty.wav"))
		.output(Some(path!("empty.wav.json")))
		.samples(800)
		.channels(Channel::VARIANTS.to_vec())
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("empty.wav.json");
	let json = json.as_object().unwrap();

	assert_eq!(json.get("left").expect("left missing").as_array().unwrap().len(), 0);
	assert_eq!(json.get("right").expect("right missing").as_array().unwrap().len(), 0);
	assert_eq!(json.get("mid").expect("mid missing").as_array().unwrap().len(), 0);
	assert_eq!(json.get("side").expect("side missing").as_array().unwrap().len(), 0);
	assert_eq!(json.get("min").expect("min missing").as_array().unwrap().len(), 0);
	assert_eq!(json.get("max").expect("max missing").as_array().unwrap().len(), 0);
}
