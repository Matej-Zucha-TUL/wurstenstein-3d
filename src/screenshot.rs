use glow::*;
use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use log::info;
use winit::dpi::PhysicalSize;

pub fn take_screenshot(gl: &Context, size: PhysicalSize<u32>) {
	let width = size.width as usize;
	let height = size.height as usize;

	let mut buf = vec![0; width * height * 4];

	unsafe {
		gl.read_pixels(
			0,
			0,
			width as i32,
			height as i32,
			RGBA,
			UNSIGNED_BYTE,
			PixelPackData::Slice(Some(buf.as_mut_slice())),
		);
	}

	std::thread::spawn(move || {
		let mut out_file = std::fs::File::create("screenshot.png").unwrap();

		// Reverse the rows so that the image is not upside down

		let buf = buf
			.chunks(width * 4)
			.rev()
			.flatten()
			.copied()
			.collect::<Vec<_>>();

		PngEncoder::new(&mut out_file)
			.write_image(
				buf.as_slice(),
				width as u32,
				height as u32,
				ExtendedColorType::Rgba8,
			)
			.unwrap();

		info!("Screenshot saved!");
	});
}
