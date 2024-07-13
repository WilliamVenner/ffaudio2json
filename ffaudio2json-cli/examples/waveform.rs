use ffaudio2json::{Channel, IntoEnumIterator, VariantArray};
use std::{fs::File, io::BufReader, path::PathBuf};

const SAMPLE_WIDTH: u32 = 20;
const SAMPLE_HEIGHT: u32 = 1000;
const SAMPLE_GAP: u32 = 2;
const SAMPLE_COLOR: image::Rgb<u8> = image::Rgb([0, 0, 0]);

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let path = PathBuf::from(std::env::args_os().nth(1).expect("no input file given"));

	let data = serde_json::from_reader::<_, serde_json::Value>(BufReader::new(File::open(&path)?))?;
	let data = data.as_object().unwrap();
	let data = Channel::iter()
		.filter_map(|channel| data.get(&channel.to_string()))
		.map(|samples| {
			samples
				.as_array()
				.unwrap()
				.iter()
				.map(|value| value.as_f64().unwrap())
				.collect::<Vec<_>>()
		})
		.collect::<Vec<_>>();

	if data.is_empty() {
		eprintln!("No data found in file");
		std::process::exit(1);
	}

	let mut image: image::ImageBuffer<image::Rgb<u8>, _> = image::ImageBuffer::new(
		(data[0].len() as u32 * SAMPLE_WIDTH) + (data[0].len().saturating_sub(1) as u32 * SAMPLE_GAP),
		(SAMPLE_HEIGHT * data.len() as u32) + (data.len().saturating_sub(1) as u32 * SAMPLE_GAP),
	);

	image.fill(255);

	for (j, samples) in data.into_iter().enumerate() {
		for (i, sample) in samples.into_iter().enumerate() {
			assert!(
				sample >= 0.0 && sample <= 1.0,
				"sample {} out of range in {} channel: {}",
				i,
				Channel::VARIANTS[j],
				sample
			);

			let i = i as u32;

			let sample = (sample * SAMPLE_HEIGHT as f64 * 0.75).ceil() as u32;

			for y in 0..sample {
				for x in 0..SAMPLE_WIDTH {
					image.put_pixel(
						(i * SAMPLE_WIDTH) + (i * SAMPLE_GAP) + x,
						(((SAMPLE_HEIGHT as f64 * 0.5) + (sample as f64 * 0.5)) as u32 - y - 1)
							+ (j as u32 * SAMPLE_HEIGHT) + (j as u32 * SAMPLE_GAP),
						SAMPLE_COLOR,
					);
				}
			}
		}
	}

	image.save("waveform.png")?;

	Ok(())
}
