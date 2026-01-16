//! Example: Read HEIC metadata using libheif-rs
//!
//! Usage: cargo run --example heic_metadata_libheif <heic_image_path>
//!
//! This example demonstrates reading HEIC metadata (dimensions, EXIF, etc.)
//! using the native libheif-rs API.

use libheif_rs::{HeifContext, ItemId};
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <heic_image_path>", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        std::process::exit(1);
    }

    println!("=== HEIC Metadata Reader (libheif-rs) ===");
    println!("File: {}", path.display());
    println!();

    // Open HEIC file using native libheif-rs API
    let path_str = path.to_string_lossy();
    println!("Reading HEIC file...");

    let ctx = match HeifContext::read_from_file(&path_str) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Failed to read HEIC file: {}", e);
            return;
        }
    };

    let handle = match ctx.primary_image_handle() {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("Failed to get primary image: {}", e);
            return;
        }
    };

    // Basic image info
    println!("--- Basic Image Info ---");
    println!("Dimensions: {} x {}", handle.width(), handle.height());
    println!("Has alpha: {}", handle.has_alpha_channel());
    println!();

    // Color profile
    println!("--- Color Profile ---");
    if let Some(nclx) = handle.color_profile_nclx() {
        println!("NCLX color profile found:");
        println!("  Matrix coefficients: {:?}", nclx.matrix_coefficients());
        println!("  Color primaries: {:?}", nclx.color_primaries());
        println!("  Transfer characteristics: {:?}", nclx.transfer_characteristics());
    } else {
        println!("No NCLX color profile");
    }
    println!();

    // EXIF data using native libheif-rs API
    println!("--- EXIF Data (via metadata_block_ids) ---");
    let mut meta_ids: Vec<ItemId> = vec![0; 1];
    let count = handle.metadata_block_ids(&mut meta_ids, b"Exif");

    if count == 0 {
        println!("No EXIF data found!");
    } else {
        println!("Found {} EXIF block(s)\n", count);

        for i in 0..count {
            let meta_id = meta_ids[i];
            println!("[EXIF #{}] Item ID: {}", i, meta_id);

            // Get raw data
            match handle.metadata(meta_id) {
                Ok(raw_data) => {
                    println!("  Size: {} bytes", raw_data.len());

                    // Show first 16 bytes as hex
                    let preview_len = std::cmp::min(raw_data.len(), 16);
                    println!("  First {} bytes: {:02X?}", preview_len, &raw_data[..preview_len]);

                    // Try to parse as EXIF (skip first 4 bytes offset)
                    if raw_data.len() > 4 {
                        let exif_data = &raw_data[4..];
                        println!("  Parsing EXIF ({} bytes)...", exif_data.len());
                        parse_exif_bytes(exif_data);
                    }
                }
                Err(e) => {
                    println!("  Failed to read metadata: {}", e);
                }
            }
            println!();
        }
    }

    // All metadata blocks
    println!("--- All Metadata Blocks ---");
    let all_metadata = handle.all_metadata();

    if all_metadata.is_empty() {
        println!("No metadata blocks found!");
    } else {
        println!("Found {} metadata block(s):", all_metadata.len());
        for (i, meta) in all_metadata.iter().enumerate() {
            println!("  [{}] Type: {:?} ({} bytes)",
                i, meta.item_type, meta.raw_data.len());
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
            println!("  Successfully parsed {} EXIF fields:", exif.fields().count());
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
                println!("  No key EXIF fields found (may be normal for some files)");
            }
        }
        Err(e) => {
            println!("  Failed to parse EXIF: {}", e);
        }
    }
}
