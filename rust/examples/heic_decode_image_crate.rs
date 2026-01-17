//! Example: Decode HEIC to JPEG using image crate with libheif-rs integration
//!
//! Usage: cargo run --example heic_decode_image <heic_image_path> [output_path]
//!
//! This example demonstrates using libheif-rs with image crate integration
//! to decode HEIC files and convert them to JPEG format.

use image::ImageReader;
use libheif_rs::integration::image::register_all_decoding_hooks;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <heic_image_path> [output_path]", args[0]);
        eprintln!("  If output_path is not provided, saves as <input>_output.jpg");
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);

    if !input_path.exists() {
        eprintln!("File not found: {}", input_path.display());
        std::process::exit(1);
    }

    println!("=== HEIC to JPEG Converter (image crate + libheif-rs) ===");
    println!("File: {}", input_path.display());
    println!();

    // Register libheif-rs decoding hooks with image crate
    println!("Registering HEIC/HEIF/AVIF decoding hooks...");
    register_all_decoding_hooks();
    println!("HEIC decoding hooks registered.");

    // Open HEIC file using image crate
    // Note: The image crate will use the registered hooks to decode HEIC files
    println!("\nOpening: {}", input_path.display());

    // Try to decode directly - the decoder hook should work based on file content
    println!("Decoding HEIC...");

    let reader = match ImageReader::open(input_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to open image: {}", e);
            return;
        }
    };

    let image = match reader.decode() {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to decode image: {}", e);
            return;
        }
    };

    println!("Decoded image:");
    println!("  Dimensions: {} x {}", image.width(), image.height());
    println!("  Color type: {:?}", image.color());

    // Convert to RGB8 for JPEG encoding
    println!("\nConverting to RGB...");
    let rgb_image = image.to_rgb8();

    // Determine output path
    let output_path = if args.len() == 3 {
        Path::new(&args[2]).to_path_buf()
    } else {
        let mut output = input_path.file_stem().unwrap().to_os_string();
        output.push("_output.jpg");
        input_path.parent().unwrap().join(output)
    };

    // Save as JPEG
    println!("\nSaving to: {}", output_path.display());

    match rgb_image.save_with_format(&output_path, image::ImageFormat::Jpeg) {
        Ok(_) => {
            println!("Successfully saved JPEG!");
        }
        Err(e) => {
            eprintln!("Failed to save image: {}", e);
            return;
        }
    }

    // Get file size
    let output_metadata = std::fs::metadata(&output_path).unwrap();
    println!("Output file size: {} bytes", output_metadata.len());
}
