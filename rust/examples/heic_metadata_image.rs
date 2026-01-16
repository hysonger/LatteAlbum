//! Example: Read HEIC metadata using image crate with libheif-rs integration
//!
//! Usage: cargo run --example heic_metadata_image <heic_image_path>
//!
//! This example demonstrates reading HEIC metadata using the image crate's
//! ImageDecoder trait, which provides access to EXIF, XMP, IPTC, and other metadata.

use image::{ImageDecoder, ImageReader};
use libheif_rs::integration::image::register_all_decoding_hooks;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <heic_image_path>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    if !input_path.exists() {
        eprintln!("File not found: {}", input_path.display());
        std::process::exit(1);
    }

    println!("=== HEIC Metadata Reader (image crate + libheif-rs) ===");
    println!("File: {}", input_path.display());
    println!();

    // Register libheif-rs decoding hooks with image crate
    println!("Registering HEIC/HEIF/AVIF decoding hooks...");
    register_all_decoding_hooks();
    println!("HEIC decoding hooks registered.");
    println!();

    // Open HEIC file using image crate
    println!("Opening: {}", input_path.display());

    let reader = match ImageReader::open(input_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to open image: {}", e);
            return;
        }
    };

    // Convert to decoder to access metadata
    println!("Creating decoder...");

    let mut decoder = match reader.into_decoder() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to create decoder: {}", e);
            return;
        }
    };

    // Read metadata using ImageDecoder trait methods
    println!("\n--- Basic Image Info ---");
    let (width, height) = decoder.dimensions();
    println!("Dimensions: {} x {}", width, height);
    println!("Color type: {:?}", decoder.color_type());

    // Orientation
    println!("\n--- Orientation ---");
    match decoder.orientation() {
        Ok(orientation) => println!("Orientation: {:?}", orientation),
        Err(e) => println!("Failed to get orientation: {}", e),
    }

    // EXIF metadata
    println!("\n--- EXIF Metadata ---");
    match decoder.exif_metadata() {
        Ok(Some(exif_data)) => {
            println!("EXIF data found: {} bytes", exif_data.len());

            // Parse EXIF using kamadak-exif
            // Note: HEIC EXIF data may need to skip the first 4 bytes (TIFF offset)
            let exif_bytes = if exif_data.len() > 4 && exif_data[0..4] == [0, 0, 0, 0] {
                println!("Skipping 4-byte TIFF offset header...");
                &exif_data[4..]
            } else {
                &exif_data
            };

            parse_exif_bytes(exif_bytes);
        }
        Ok(None) => {
            println!("No EXIF data found");
        }
        Err(e) => {
            println!("Failed to read EXIF: {}", e);
        }
    }

    // XMP metadata
    println!("\n--- XMP Metadata ---");
    match decoder.xmp_metadata() {
        Ok(Some(xmp_data)) => {
            println!("XMP data found: {} bytes", xmp_data.len());
            println!("First 200 bytes: {:?}", String::from_utf8_lossy(&xmp_data[..std::cmp::min(200, xmp_data.len())]));
        }
        Ok(None) => {
            println!("No XMP data found");
        }
        Err(e) => {
            println!("Failed to read XMP: {}", e);
        }
    }

    // IPTC metadata
    println!("\n--- IPTC Metadata ---");
    match decoder.iptc_metadata() {
        Ok(Some(iptc_data)) => {
            println!("IPTC data found: {} bytes", iptc_data.len());
        }
        Ok(None) => {
            println!("No IPTC data found");
        }
        Err(e) => {
            println!("Failed to read IPTC: {}", e);
        }
    }

    // ICC Profile
    println!("\n--- ICC Profile ---");
    match decoder.icc_profile() {
        Ok(Some(icc_data)) => {
            println!("ICC profile found: {} bytes", icc_data.len());
        }
        Ok(None) => {
            println!("No ICC profile found");
        }
        Err(e) => {
            println!("Failed to read ICC profile: {}", e);
        }
    }
}

/// Parse EXIF bytes and print important tags
fn parse_exif_bytes(data: &[u8]) {
    use exif::Reader;

    let mut cursor = std::io::Cursor::new(data);
    let exif_reader = Reader::new();

    match exif_reader.read_from_container(&mut cursor) {
        Ok(exif) => {
            println!("Successfully parsed {} EXIF fields:", exif.fields().count());
            println!();

            // Print important fields
            let mut found = false;

            for field in exif.fields() {
                let tag = field.tag;
                let value_str = field.value.display_as(tag).to_string();

                match tag {
                    exif::Tag::Make => {
                        println!("  Make: {}", value_str.trim_matches('"'));
                        found = true;
                    }
                    exif::Tag::Model => {
                        println!("  Model: {}", value_str.trim_matches('"'));
                        found = true;
                    }
                    exif::Tag::LensModel => {
                        println!("  LensModel: {}", value_str.trim_matches('"'));
                        found = true;
                    }
                    exif::Tag::DateTimeOriginal => {
                        println!("  DateTimeOriginal: {}", value_str.trim_matches('"'));
                        found = true;
                    }
                    exif::Tag::ExposureTime => {
                        println!("  ExposureTime: {}", value_str.trim_matches('"'));
                        found = true;
                    }
                    exif::Tag::FNumber => {
                        println!("  FNumber: {}", value_str.trim_matches('"'));
                        found = true;
                    }
                    exif::Tag::ISOSpeed => {
                        if let Ok(iso) = value_str.trim_matches('"').parse::<i32>() {
                            println!("  ISOSpeed: {}", iso);
                        }
                        found = true;
                    }
                    exif::Tag::FocalLength => {
                        println!("  FocalLength: {}", value_str.trim_matches('"'));
                        found = true;
                    }
                    _ => {}
                }
            }

            if !found {
                println!("  No key EXIF fields found");
            }
        }
        Err(e) => {
            println!("  Failed to parse EXIF: {}", e);
        }
    }
}
