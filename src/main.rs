use std::path::PathBuf;

use image::{codecs::png::PngEncoder, DynamicImage, GenericImageView, ImageBuffer, Rgb};

fn main() {
	let mut args = std::env::args();
	let mode = match args.nth(1).expect("Missing chromacy argument").as_str() {
		"1" => ColourBlindness::Monochromacy,
		"2" => {
			let missing_colour = Colour::parse(&args.next().expect("Missing colour argument"));
			ColourBlindness::Dichromacy(missing_colour)
		}
		_ => panic!("Unexpected chromacy argument"),
	};
	let filter_name = args.next().expect("Missing filter argument");
	let image_path = PathBuf::from(args.next().expect("Missing image argument"));
	let image_file_stem = image_path
		.file_stem()
		.expect("Image path had no file name")
		.to_str()
		.unwrap();
	let image = image::open(&image_path).unwrap().into_rgb8();
	match mode {
		ColourBlindness::Monochromacy => {
			apply_monochromacy_filter(&filter_name, image, image_file_stem, args.next());
		}
		ColourBlindness::Dichromacy(missing_colour) => {
			apply_dichromacy_filter(
				&filter_name,
				image,
				image_file_stem,
				args.next(),
				missing_colour,
			);
		}
	};
}

fn apply_monochromacy_filter(
	filter_name: &str,
	mut image: ImageBuffer<Rgb<u8>, Vec<u8>>,
	image_file_stem: &str,
	output_path: Option<String>,
) {
	let output_path =
		output_path.unwrap_or_else(|| format!("{filter_name} - {image_file_stem}.png",));
	let filter = MonochromacyFilter::load(filter_name);
	filter.apply_image(&mut image);
	DynamicImage::ImageRgb8(image)
		.into_luma8()
		.write_with_encoder(PngEncoder::new_with_quality(
			std::fs::File::create(output_path).unwrap(),
			image::codecs::png::CompressionType::Best,
			image::codecs::png::FilterType::Adaptive,
		))
		.unwrap();
}

fn apply_dichromacy_filter(
	filter_name: &str,
	mut image: ImageBuffer<Rgb<u8>, Vec<u8>>,
	image_file_stem: &str,
	output_path: Option<String>,
	missing_colour: Colour,
) {
	let output_path = output_path
		.unwrap_or_else(|| format!("{filter_name} - {missing_colour} - {image_file_stem}.png",));
	let filter = DichromacyFilter::load(filter_name, missing_colour);
	filter.apply_image(&mut image);
	DynamicImage::ImageRgb8(image)
		.write_with_encoder(PngEncoder::new_with_quality(
			std::fs::File::create(output_path).unwrap(),
			image::codecs::png::CompressionType::Best,
			image::codecs::png::FilterType::Adaptive,
		))
		.unwrap();
}

#[derive(Clone, Copy)]
pub enum Colour {
	Red,
	Green,
	Blue,
}

#[derive(Clone, Copy)]
pub enum ColourBlindness {
	Monochromacy,
	Dichromacy(Colour),
}

pub struct MonochromacyFilter(DynamicImage);

impl MonochromacyFilter {
	pub fn load(name: &str) -> Self {
		let filter = image::io::Reader::open(format!("filters/{name}"))
			.unwrap()
			.decode()
			.unwrap();
		Self(filter)
	}
	fn apply_pixel(&self, x: u32, y: u32, colour: Rgb<u8>) -> Rgb<u8> {
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
	pub fn apply_image(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
		for (x, y, colour) in image.enumerate_pixels_mut() {
			*colour = self.apply_pixel(x, y, *colour);
		}
	}
}

impl Colour {
	pub fn parse(text: &str) -> Self {
		match text {
			"red" => Self::Red,
			"green" => Self::Green,
			"blue" => Self::Blue,
			_ => panic!("Unexpected colour argument {text}"),
		}
	}
}

impl std::fmt::Display for Colour {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Red => f.write_str("red"),
			Self::Green => f.write_str("green"),
			Self::Blue => f.write_str("blue"),
		}
	}
}

pub struct DichromacyFilter {
	filter: DynamicImage,
	missing_colour: Colour,
}

impl DichromacyFilter {
	pub fn load(name: &str, missing_colour: Colour) -> Self {
		let filter = image::io::Reader::open(format!("filters/{name}"))
			.unwrap()
			.decode()
			.unwrap();
		Self {
			filter,
			missing_colour,
		}
	}
	fn apply_pixel(&self, x: u32, y: u32, colour: Rgb<u8>) -> Rgb<u8> {
		let pixel = self
			.filter
			.get_pixel(x % self.filter.width(), y % self.filter.height());
		if pixel.0[0] == 0 {
			Rgb([colour[self.missing_colour as usize]; 3])
		} else {
			colour
			// Rgb(std::array::from_fn(|i| {
			// 	if i == self.missing_colour as usize {
			// 		(colour
			// 			.0
			// 			.iter()
			// 			.enumerate()
			// 			.filter_map(|(i, v)| {
			// 				(i != self.missing_colour as usize).then_some(*v as u16)
			// 			})
			// 			.sum::<u16>() / 2) as u8
			// 	} else {
			// 		colour.0[i]
			// 	}
			// }))
		}
	}
	pub fn apply_image(&self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
		for (x, y, colour) in image.enumerate_pixels_mut() {
			*colour = self.apply_pixel(x, y, *colour);
		}
	}
}

fn multiply(a: u8, b: u8) -> u8 {
	(a as f32 / 255.0 * (b as f32 / 255.0) * 255.0) as u8
}
