//! Profile memory allocation in JPEG transcoding
//!
//! Usage: cargo run --example profile_memory <jpg_path>
//!
//! This example profiles the memory allocation during JPEG decode and resize.

use image::ImageReader;
use std::path::Path;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <jpg_path>", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        std::process::exit(1);
    }

    println!("=== Memory Profile: {:?} ===\n", path.file_name().unwrap());

    // Profile 1: Just open and get info
    println!("1. ImageReader::open() + header parse only");
    for i in 0..3 {
        let start = Instant::now();
        let reader = ImageReader::open(path).unwrap();
        let _ = reader.format();
        let _ = reader.into_decoder().unwrap();
        let elapsed = start.elapsed();
        println!("   Run {}: {:.2}ms", i + 1, elapsed.as_secs_f64() * 1000.0);
    }
    println!();

    // Profile 2: Decode only (full size)
    println!("2. decode() - full size decode");
    for i in 0..3 {
        let start = Instant::now();
        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let elapsed = start.elapsed();
        println!("   Run {}: {:.2}ms, {}x{}",
            i + 1,
            elapsed.as_secs_f64() * 1000.0,
            img.width(),
            img.height()
        );
    }
    println!();

    // Profile 3: to_rgb8() only
    println!("3. to_rgb8() - after full decode");
    for i in 0..3 {
        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let start = Instant::now();
        let _rgb = img.to_rgb8();
        let elapsed = start.elapsed();
        println!("   Run {}: {:.2}ms", i + 1, elapsed.as_secs_f64() * 1000.0);
    }
    println!();

    // Profile 4: Resize only (no decode)
    println!("4. resize() only - decode + resize");
    for i in 0..3 {
        let start = Instant::now();
        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let ratio = img.height() as f64 / img.width() as f64;
        let height = (450u32 as f64 * ratio) as u32;
        let resized = img.resize(450, height, image::imageops::FilterType::Lanczos3);
        let _rgb = resized.to_rgb8();
        let elapsed = start.elapsed();
        println!("   Run {}: {:.2}ms", i + 1, elapsed.as_secs_f64() * 1000.0);
    }
    println!();

    // Profile 5: Different resize sizes
    println!("5. Different resize sizes:");
    for size in &[150, 300, 450, 600, 900] {
        let start = Instant::now();
        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let ratio = img.height() as f64 / img.width() as f64;
        let height = (*size as f64 * ratio) as u32;
        let _ = img.resize(*size, height, image::imageops::FilterType::Lanczos3);
        let elapsed = start.elapsed();
        println!("   {}x{}: {:.2}ms", *size, height, elapsed.as_secs_f64() * 1000.0);
    }
}
