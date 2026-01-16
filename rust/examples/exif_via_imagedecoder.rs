//! Example: Extract EXIF using ImageDecoder trait
//!
//! Usage: cargo run --example exif_via_imagedecoder <image_path>
//!
//! This example demonstrates using the ImageDecoder trait's exif_metadata()
//! method to extract EXIF information from images.

use exif::Reader;
use image::ImageDecoder;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <image_path>", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        std::process::exit(1);
    }

    println!("=== EXIF via ImageDecoder ===");
    println!("File: {}", path.display());
    println!();

    // Open image and get decoder
    let reader = match image::ImageReader::open(path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to open image: {}", e);
            return;
        }
    };

    // Convert to decoder to access metadata methods
    let mut decoder = match reader.into_decoder() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to create decoder: {}", e);
            return;
        }
    };

    // Get EXIF data via ImageDecoder trait
    match decoder.exif_metadata() {
        Ok(Some(exif_data)) => {
            println!("EXIF data found: {} bytes", exif_data.len());

            // HEIC/JPEG EXIF data has a 4-byte TIFF offset header
            let exif_bytes = if exif_data.len() > 4 && exif_data[0..4] == [0, 0, 0, 0] {
                println!("Skipping 4-byte TIFF offset header...");
                &exif_data[4..]
            } else {
                &exif_data
            };

            parse_and_print_exif(exif_bytes);
        }
        Ok(None) => {
            println!("No EXIF data found");
        }
        Err(e) => {
            println!("Failed to read EXIF: {}", e);
        }
    }
}

/// Parse EXIF bytes and print important tags
fn parse_and_print_exif(data: &[u8]) {
    let mut cursor = std::io::Cursor::new(data);
    let exif_reader = Reader::new();

    match exif_reader.read_from_container(&mut cursor) {
        Ok(exif) => {
            println!("\nSuccessfully parsed {} EXIF fields:", exif.fields().count());
            println!();

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
