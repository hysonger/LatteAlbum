//! Example: Extract EXIF metadata using kamadak-exif
//!
//! Usage: cargo run --example exif_kamadak <image_path>
//!
//! This example demonstrates extracting all EXIF tags from JPEG/HEIC files
//! using the kamadak-exif library.

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

    println!("=== kamadak-exif EXIF extraction ===");
    println!("File: {}", path.display());
    println!();

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return;
        }
    };

    let exif_reader = exif::Reader::new();
    let exif = match exif_reader.read_from_container(&mut std::io::BufReader::new(file)) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to parse EXIF: {}", e);
            return;
        }
    };

    println!("Found {} EXIF fields:\n", exif.fields().count());

    for field in exif.fields() {
        let tag = field.tag;
        let value_str = field.value.display_as(tag).to_string();

        // Detect potential quote issues
        let has_quotes = (value_str.starts_with('"') && value_str.ends_with('"'))
            || (value_str.starts_with('\'') && value_str.ends_with('\''));

        println!(
            "[{:>3}] {:<30} = {} {}",
            tag.number(),
            format!("{:?}", tag),
            if has_quotes { "HAS_QUOTES" } else { "" },
            value_str
        );
    }
}
