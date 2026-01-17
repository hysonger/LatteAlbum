//! Example: Extract EXIF using little_exif library
//!
//! Usage: cargo run --example exif_via_little_exif <image_path>
//!
//! This example demonstrates using the little_exif library to extract
//! EXIF information from images (JPEG, PNG, HEIC, TIFF, WebP, etc.)

use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
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

    println!("=== EXIF via little_exif ===");
    println!("File: {}", path.display());
    println!();

    // Load metadata from file
    let metadata = match Metadata::new_from_path(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to read metadata: {}", e);
            std::process::exit(1);
        }
    };

    // Count total tags across all IFDs
    let total_tags: usize = metadata.get_ifds().iter().map(|ifd| ifd.get_tags().len()).sum();
    println!("Found {} EXIF tags across {} IFD(s)", total_tags, metadata.get_ifds().len());

    // List all tags with their values
    println!();

    // Group tags by IFD
    for ifd in metadata.get_ifds() {
        let ifd_type = ifd.get_ifd_type();
        let ifd_nr = ifd.get_generic_ifd_nr();
        println!("--- IFD: {:?} (nr={}) ---", ifd_type, ifd_nr);

        for tag in ifd.get_tags() {
            print_tag(tag);
        }
        println!();
    }
}

/// Print a tag with its value in a readable format
fn print_tag(tag: &ExifTag) {
    // Use Debug format to print all tag info
    println!("    {:?}", tag);
}
