//! Example: Extract EXIF metadata using libheif-rs
//!
//! Usage: cargo run --example exif_libheif <heic_image_path>
//!
//! This example demonstrates extracting EXIF metadata from HEIC files
//! using the libheif-rs library, then parsing with kamadak-exif.

use libheif_rs::HeifContext;
use std::env::args;
use std::path::Path;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <heic_image_path>", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        std::process::exit(1);
    }

    println!("=== libheif-rs EXIF extraction ===");
    println!("File: {}", path.display());
    println!();

    // Open HEIC file with libheif
    let path_str = path.to_string_lossy();
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

    println!("Image dimensions: {} x {}", handle.width(), handle.height());
    println!();

    // Extract all metadata blocks
    println!("--- All metadata blocks from libheif-rs ---");
    let all_metadata = handle.all_metadata();

    if all_metadata.is_empty() {
        println!("No metadata found!");
    } else {
        println!("Found {} metadata block(s):\n", all_metadata.len());

        for (i, meta) in all_metadata.iter().enumerate() {
            println!("[{}] Type: {:?}", i, meta.item_type);
            println!("    Content-Type: {}", meta.content_type);
            println!("    URI-Type: {}", meta.uri_type);
            println!("    Data size: {} bytes", meta.raw_data.len());

            // Show first 64 bytes
            let preview_len = std::cmp::min(meta.raw_data.len(), 64);
            println!("    First {} bytes: {:02X?}", preview_len, &meta.raw_data[..preview_len]);
            println!();
        }
    }

    // Find EXIF data and parse with kamadak-exif
    println!("--- Parsing EXIF with kamadak-exif ---");

    for meta in &all_metadata {
        // Check if this is EXIF data (item_type should be "Exif")
        let type_str = format!("{}", meta.item_type);
        if type_str != "Exif" {
            continue;
        }

        println!("Found EXIF block, size: {} bytes", meta.raw_data.len());
        println!();

        // Show detailed byte analysis
        println!("=== Byte analysis ===");
        if meta.raw_data.len() >= 8 {
            // Show first 8 bytes as different interpretations
            println!("First 8 bytes:");
            println!("  As hex: {:02X?} ", &meta.raw_data[..8]);
            println!("  As u32 (be): {}", u32::from_be_bytes([
                meta.raw_data[0], meta.raw_data[1], meta.raw_data[2], meta.raw_data[3]
            ]));
            println!("  As u32 (le): {}", u32::from_le_bytes([
                meta.raw_data[0], meta.raw_data[1], meta.raw_data[2], meta.raw_data[3]
            ]));

            // Check for TIFF magic number (0x002A for big-endian, 0x2A00 for little-endian)
            let be_magic = u16::from_be_bytes([meta.raw_data[0], meta.raw_data[1]]);
            let le_magic = u16::from_le_bytes([meta.raw_data[0], meta.raw_data[1]]);
            println!("  TIFF magic (be): 0x{:04X} ({})", be_magic,
                if be_magic == 0x002A { "big-endian" } else if be_magic == 0x2A00 { "little-endian" } else { "unknown" });
            println!("  TIFF magic (le): 0x{:04X}", le_magic);
        }
        println!();

        // Try different parsing strategies
        println!("=== Trying different parsing strategies ===\n");

        // Strategy 1: Skip first 4 bytes (standard HEIF EXIF offset)
        println!("Strategy 1: Skip first 4 bytes (standard offset)");
        parse_exif_with_strategy(&meta.raw_data, 4, "standard");

        // Strategy 2: Skip first 8 bytes (some implementations)
        if meta.raw_data.len() > 8 {
            println!("\nStrategy 2: Skip first 8 bytes");
            parse_exif_with_strategy(&meta.raw_data, 8, "offset8");
        }

        // Strategy 3: Try without skipping (raw data)
        println!("\nStrategy 3: Try without skipping (raw data)");
        parse_exif_with_strategy(&meta.raw_data, 0, "raw");

        // Strategy 4: Skip based on the offset value in the first 4 bytes
        let offset = u32::from_be_bytes([
            meta.raw_data[0], meta.raw_data[1], meta.raw_data[2], meta.raw_data[3]
        ]);
        if offset > 0 && offset < meta.raw_data.len() as u32 {
            println!("\nStrategy 4: Skip {} bytes (from offset field)", offset);
            parse_exif_with_strategy(&meta.raw_data, offset as usize, "from_offset");
        }

        // Only process the first EXIF block
        break;
    }
}

fn parse_exif_with_strategy(data: &[u8], skip: usize, strategy_name: &str) {
    if data.len() <= skip {
        println!("  [{}] Data too small ({} bytes, need to skip {})", strategy_name, data.len(), skip);
        return;
    }

    let exif_data = &data[skip..];
    println!("  [{}] Trying to parse {} bytes...", strategy_name, exif_data.len());

    use exif::Reader;
    let mut cursor = std::io::Cursor::new(exif_data);
    let exif_reader = Reader::new();

    match exif_reader.read_from_container(&mut cursor) {
        Ok(exif) => {
            println!("  [{}] SUCCESS! Found {} EXIF fields:", strategy_name, exif.fields().count());

            // Group and print important fields
            let mut found_important = false;

            // Camera info
            for field in exif.fields() {
                let tag = field.tag;
                let value_str = field.value.display_as(tag).to_string();

                // Only print interesting tags
                match tag {
                    exif::Tag::Make | exif::Tag::Model |
                    exif::Tag::LensModel |
                    exif::Tag::DateTimeOriginal | exif::Tag::DateTimeDigitized |
                    exif::Tag::ExposureTime | exif::Tag::FNumber |
                    exif::Tag::ISOSpeed | exif::Tag::FocalLength => {
                        if !found_important {
                            println!("    --- Important fields ---");
                            found_important = true;
                        }
                        println!("    [{:>3}] {:<25} = {}",
                            tag.number(),
                            format!("{:?}", tag),
                            value_str.trim_matches('"').trim_matches('\'')
                        );
                    }
                    _ => {}
                }
            }

            if found_important {
                println!();
            }

            // Print all fields if few, otherwise summarize
            let total = exif.fields().count();
            if total <= 20 {
                println!("    --- All {} fields ---", total);
                for field in exif.fields() {
                    let tag = field.tag;
                    let value_str = field.value.display_as(tag).to_string();
                    let has_quotes = (value_str.starts_with('"') && value_str.ends_with('"'))
                        || (value_str.starts_with('\'') && value_str.ends_with('\''));
                    println!("    [{:>3}] {:<25} = {}{}",
                        tag.number(),
                        format!("{:?}", tag),
                        if has_quotes { "⚠️ " } else { "" },
                        value_str
                    );
                }
            } else {
                println!("    ... and {} more fields (total {})", total - 10, total);
            }
        }
        Err(e) => {
            println!("  [{}] FAILED: {}", strategy_name, e);
        }
    }
}
