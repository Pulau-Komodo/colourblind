use std::{env::args, path::PathBuf};

use image::{codecs::png::PngEncoder, DynamicImage, GenericImageView, Rgb};

fn main() {
	let pattern_name = args().nth(1).expect("Missing pattern argument");
	let image_path = PathBuf::from(args().nth(2).expect("Missing image argument"));
	let output_path = args().nth(3).unwrap_or_else(|| {
		format!(
			"{pattern_name} - {}.png",
			image_path
				.file_stem()
				.expect("Image path had no file name")
				.to_str()
				.unwrap()
		)
	});
	let patterns = ChannelPatterns::load(&pattern_name);
	let mut image = image::open(image_path).unwrap().into_rgb8();
	for (x, y, colour) in image.enumerate_pixels_mut() {
		*colour = patterns.apply(x, y, *colour);
	}
	DynamicImage::ImageRgb8(image)
		.into_luma8()
		.write_with_encoder(PngEncoder::new_with_quality(
			std::fs::File::create(output_path).unwrap(),
			image::codecs::png::CompressionType::Best,
			image::codecs::png::FilterType::Adaptive,
		))
		.unwrap();
}

struct ChannelPatterns(DynamicImage);

impl ChannelPatterns {
	fn load(name: &str) -> Self {
		let patterns = image::io::Reader::open(format!("patterns/{name}"))
			.unwrap()
			.decode()
			.unwrap();
		Self(patterns)
	}
	fn apply(&self, x: u32, y: u32, colour: Rgb<u8>) -> Rgb<u8> {
		let pixel = self.0.get_pixel(x % self.0.width(), y % self.0.height());
		let intensity = colour
			.0
			.into_iter()
			.zip(pixel.0)
			.map(|(a, b)| multiply(a, b))
			.max()
			.unwrap();
		Rgb([intensity, intensity, intensity])
	}
}

fn multiply(a: u8, b: u8) -> u8 {
	(a as f32 / 255.0 * (b as f32 / 255.0) * 255.0) as u8
}
