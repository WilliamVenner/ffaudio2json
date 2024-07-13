use std::{fs::File, io::BufReader, path::PathBuf};

const SAMPLE_WIDTH: u32 = 20;
const SAMPLE_HEIGHT: u32 = 1000;
const SAMPLE_GAP: u32 = 2;
const SAMPLE_COLOR: image::Rgb<u8> = image::Rgb([0, 0, 0]);

#[derive(serde::Deserialize)]
struct Output {
	#[serde(rename = "mid")]
	samples: Vec<f64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let path = PathBuf::from(std::env::args_os().nth(1).expect("no input file given"));

	let samples = serde_json::from_reader::<_, Output>(BufReader::new(File::open(&path)?))?.samples;

	let mut image: image::ImageBuffer<image::Rgb<u8>, _> = image::ImageBuffer::new(
		(samples.len() as u32 * SAMPLE_WIDTH) + (samples.len().saturating_sub(1) as u32 * SAMPLE_GAP),
		SAMPLE_HEIGHT,
	);

	image.fill(255);

	for (i, sample) in samples.iter().copied().enumerate() {
		let i = i as u32;

		let sample = (sample * SAMPLE_HEIGHT as f64).round() as u32;

		for y in 0..sample {
			for x in 0..SAMPLE_WIDTH {
				image.put_pixel(
					(i * SAMPLE_WIDTH) + (i * SAMPLE_GAP) + x,
					((SAMPLE_HEIGHT as f64 * 0.5) + (sample as f64 * 0.5)) as u32 - y - 1,
					SAMPLE_COLOR,
				);
			}
		}
	}

	image.save("waveform.png")?;

	Ok(())
}
