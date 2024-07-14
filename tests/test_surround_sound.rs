use ffaudio2json::Channel;
use strum::VariantArray;

#[macro_use]
mod common;

#[test]
fn test_surround_sound() {
	enable_logging!();

	ffaudio2json::FfAudio2Json::builder()
		.no_header(true)
		.input(path!("DolbyAtmosDemo.aac"))
		.output(Some(path!("DolbyAtmosDemo.aac.json")))
		.samples(50000)
		.channels(Channel::VARIANTS.to_vec())
		.build()
		.unwrap()
		.run()
		.unwrap();

	let json = open_json!("DolbyAtmosDemo.aac.json");
	let json = json.as_object().unwrap();

	let left = json.get("left").expect("left missing").as_array().unwrap().len();
	let right = json.get("right").expect("right missing").as_array().unwrap().len();
	let mid = json.get("mid").expect("mid missing").as_array().unwrap().len();
	let side = json.get("side").expect("side missing").as_array().unwrap().len();
	let min = json.get("min").expect("min missing").as_array().unwrap().len();
	let max = json.get("max").expect("max missing").as_array().unwrap().len();

	assert!(left >= 40000 && left <= 50000, "40000 <= {left} <= 50000");
	assert!(right >= 40000 && right <= 50000, "40000 <= {right} <= 50000");
	assert!(mid >= 40000 && mid <= 50000, "40000 <= {mid} <= 50000");
	assert!(side >= 40000 && side <= 50000, "40000 <= {side} <= 50000");
	assert!(min >= 40000 && min <= 50000, "40000 <= {min} <= 50000");
	assert!(max >= 40000 && max <= 50000, "40000 <= {max} <= 50000");
}
