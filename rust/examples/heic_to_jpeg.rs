//! HEIC to JPEG conversion test utility
//!
//! Usage: cargo run --example heic_to_jpeg <input.heic> [output.jpg]
//!
//! Converts a HEIC/HEIF file to JPEG and saves it.

use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use std::env;
use std::path::PathBuf;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input.heic> [output.jpg]", args[0]);
        eprintln!("  input.heic: Path to HEIC/HEIF file");
        eprintln!("  output.jpg: Output JPEG path (default: same name with .jpg extension)");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from(input_path).with_extension("jpg")
    };

    println!("Input: {}", input_path);
    println!("Output: {}", output_path.display());

    // Read HEIC file
    println!("Reading HEIC file...");
    let ctx = HeifContext::read_from_file(input_path)
        .expect("Failed to read HEIC file");

    let handle = ctx.primary_image_handle()
        .expect("Failed to get primary image handle");

    println!("Image dimensions: {}x{}", handle.width(), handle.height());

    // Decode to RGBA - libheif handles YCbCr to RGBA conversion internally
    println!("Decoding to RGBA (libheif handles YCbCr -> RGB conversion)...");
    let lib_heif = LibHeif::new();
    let image = lib_heif.decode(
        &handle,
        ColorSpace::Rgb(RgbChroma::Rgba),
        None,
    ).expect("Failed to decode image");

    println!("Decoded image: {}x{}", image.width(), image.height());

    // Get interleaved RGBA data
    let planes = image.planes();
    let interleaved = planes.interleaved
        .as_ref()
        .expect("No interleaved plane available");

    println!("Interleaved: {}x{}, stride={}, channels=4",
             interleaved.width, interleaved.height, interleaved.stride);

    // Handle stride vs width difference (for proper scaling)
    let width = interleaved.width;
    let height = interleaved.height;
    let stride = interleaved.stride as usize;
    let data = &interleaved.data;

    // Create RGBA image, handling stride padding if necessary
    let rgba_image = if stride == width as usize * 4 {
        // Data is tightly packed, can use directly
        image::RgbaImage::from_raw(width, height, data.to_vec())
            .expect("Failed to create image from raw data")
    } else {
        // Data has padding, need to copy row by row
        println!("Stride ({}) != width * 4 ({}) - copying with stride handling", stride, width as usize * 4);
        let mut rgb_data = Vec::with_capacity((width as usize * height as usize * 4) as usize);
        let bytes_per_row = width as usize * 4;
        for row in 0..height as usize {
            let row_offset = row * stride;
            rgb_data.extend_from_slice(&data[row_offset..row_offset + bytes_per_row]);
        }
        image::RgbaImage::from_raw(width, height, rgb_data)
            .expect("Failed to create image from raw data")
    };

    // Convert RGBA to RGB (drop alpha channel)
    let rgb_image = image::DynamicImage::ImageRgba8(rgba_image).to_rgb8();

    // Encode as JPEG
    println!("Encoding to JPEG...");
    let mut jpeg_bytes = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
        &mut jpeg_bytes,
        90, // Quality 90%
    );
    encoder.encode_image(&rgb_image)
        .expect("Failed to encode JPEG");

    // Save to file
    println!("Saving to {}...", output_path.display());
    std::fs::write(&output_path, &jpeg_bytes)
        .expect("Failed to write output file");

    println!("Success! Output size: {} bytes", jpeg_bytes.len());
}
