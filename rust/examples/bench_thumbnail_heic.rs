//! Benchmark: HEIC thumbnail methods comparison
//!
//! Usage: cargo run --example benchmark_heic_thumbnail [heic_path]
//!
//! This example compares different methods for generating HEIC thumbnails:
//! 1. libheif scale() - libheif's built-in scaling
//! 2. DynamicImage::thumbnail() - fast integer algorithm after conversion
//! 3. DynamicImage::resize(Triangle) - bilinear after conversion

use image::codecs::jpeg::JpegEncoder;
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use std::path::Path;
use std::time::{Duration, Instant};

const TARGET_SIZES: &[u32] = &[300, 450, 900, 0]; // small, medium, large, full
const RUNS: usize = 5;

#[derive(Debug, Clone)]
struct TimingResult {
    total_avg: Duration,
    decode: Duration,
    process: Duration,
    encode: Duration,
    output_size: usize,
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --example benchmark_heic_thumbnail <heic_path>");
        std::process::exit(1);
    });

    let path = Path::new(&path);
    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        std::process::exit(1);
    }

    // Get image dimensions
    let dims = get_heic_dimensions(path);
    println!("=== HEIC Thumbnail Methods Benchmark ===");
    println!("Image: {} ({}x{})", path.display(), dims.0, dims.1);
    println!("Runs per test: {}", RUNS);
    println!();

    // Benchmark each method at each target size
    for target in TARGET_SIZES {
        let target = *target; // 解引用
        let target_name = match target {
            0 => "full",
            300 => "small",
            450 => "medium",
            900 => "large",
            _ => "custom",
        };
        println!("=== Target: {} ({}px) ===", target_name, target);

        // Method 1: libheif scale() - current implementation
        let libheif_result = benchmark_libheif_scale(path, target);
        print_result("libheif scale()", &libheif_result);

        // Method 2: DynamicImage::thumbnail() - fast integer algorithm
        if target > 0 {
            let thumbnail_result = benchmark_dynamic_image_thumbnail(path, target);
            print_result("DynamicImage::thumbnail()", &thumbnail_result);

            // Calculate speedup vs libheif
            let speedup = libheif_result.total_avg.as_secs_f64() / thumbnail_result.total_avg.as_secs_f64();
            println!(
                "  thumbnail() vs libheif: {:.2}x {}",
                speedup,
                if speedup > 1.0 {
                    format!("(thumbnail faster by {:.0}%)", (speedup - 1.0) * 100.0)
                } else {
                    format!("(libheif faster by {:.0}%)", (1.0 / speedup - 1.0) * 100.0)
                }
            );
        }

        // Method 3: DynamicImage::resize(Triangle)
        if target > 0 {
            let triangle_result = benchmark_dynamic_image_resize(path, target);
            print_result("DynamicImage::resize(Triangle)", &triangle_result);
        }
        println!();
    }

    // Summary
    println!("=== Summary ===");
    println!("Method                    Small   Medium  Large");
    println!("------------------------------------------------");

    let mut libheif_times = Vec::new();
    let mut thumbnail_times = Vec::new();
    let mut triangle_times = Vec::new();

    for target in [300u32, 450, 900] {
        libheif_times.push(benchmark_libheif_scale(path, target).total_avg.as_secs_f64() * 1000.0);
        thumbnail_times.push(benchmark_dynamic_image_thumbnail(path, target).total_avg.as_secs_f64() * 1000.0);
        triangle_times.push(benchmark_dynamic_image_resize(path, target).total_avg.as_secs_f64() * 1000.0);
    }

    println!("{:<24} {:>6.0}ms {:>6.0}ms {:>6.0}ms", "libheif scale()", libheif_times[0], libheif_times[1], libheif_times[2]);
    println!("{:<24} {:>6.0}ms {:>6.0}ms {:>6.0}ms", "DynamicImage::thumbnail()", thumbnail_times[0], thumbnail_times[1], thumbnail_times[2]);
    println!("{:<24} {:>6.0}ms {:>6.0}ms {:>6.0}ms", "DynamicImage::resize(Triangle)", triangle_times[0], triangle_times[1], triangle_times[2]);
}

fn get_heic_dimensions(path: &Path) -> (u32, u32) {
    let path_str = path.to_string_lossy();
    let ctx = HeifContext::read_from_file(&path_str).unwrap();
    let handle = ctx.primary_image_handle().unwrap();
    (handle.width(), handle.height())
}

/// Method 1: libheif's built-in scale()
fn benchmark_libheif_scale(path: &Path, target_width: u32) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut process_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;

    for _ in 0..RUNS {
        let start = Instant::now();

        // Decode HEIC
        let path_str = path.to_string_lossy();
        let ctx = HeifContext::read_from_file(&path_str).unwrap();
        let handle = ctx.primary_image_handle().unwrap();
        let decode_end = start.elapsed();

        let lib_heif = LibHeif::new();
        let image = lib_heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None).unwrap();

        // Scale using libheif
        let process_start = Instant::now();
        let scaled = if target_width == 0 {
            image
        } else {
            let ratio = image.height() as f64 / image.width() as f64;
            let target_height = (target_width as f64 * ratio) as u32;
            if image.width() > target_width || image.height() > target_height {
                image.scale(target_width, target_height, None).unwrap()
            } else {
                image
            }
        };
        let process_end = process_start.elapsed();

        // Convert to RGB JPEG
        let encode_start = Instant::now();
        let (width, height, data) = get_rgba_from_heif(&scaled);
        let rgba_image = image::RgbaImage::from_raw(width, height, data).unwrap();
        let rgb_image = image::DynamicImage::ImageRgba8(rgba_image).to_rgb8();

        let mut bytes = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut bytes, 80);
        encoder.encode_image(&rgb_image).unwrap();
        let encode_end = encode_start.elapsed();

        let total_end = start.elapsed();

        decode_times.push(decode_end);
        process_times.push(process_end);
        encode_times.push(encode_end);
        total_times.push(total_end);
        output_size = bytes.len();
    }

    TimingResult {
        total_avg: avg_duration(&total_times),
        decode: avg_duration(&decode_times),
        process: avg_duration(&process_times),
        encode: avg_duration(&encode_times),
        output_size,
    }
}

/// Method 2: Convert to DynamicImage then use thumbnail()
fn benchmark_dynamic_image_thumbnail(path: &Path, target_width: u32) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut process_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;

    for _ in 0..RUNS {
        let start = Instant::now();

        // Decode HEIC
        let path_str = path.to_string_lossy();
        let ctx = HeifContext::read_from_file(&path_str).unwrap();
        let handle = ctx.primary_image_handle().unwrap();
        let decode_end = start.elapsed();

        let lib_heif = LibHeif::new();
        let image = lib_heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None).unwrap();

        // Convert to DynamicImage
        let process_start = Instant::now();
        let (width, height, data) = get_rgba_from_heif(&image);
        let rgba_image = image::RgbaImage::from_raw(width, height, data).unwrap();
        let dynamic_image = image::DynamicImage::ImageRgba8(rgba_image);

        // Use thumbnail() method
        let thumb = dynamic_image.thumbnail(target_width, target_width);
        let rgb_image = thumb.to_rgb8();
        let process_end = process_start.elapsed();

        // Encode as JPEG
        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut bytes, 80);
        encoder.encode_image(&rgb_image).unwrap();
        let encode_end = encode_start.elapsed();

        let total_end = start.elapsed();

        decode_times.push(decode_end);
        process_times.push(process_end);
        encode_times.push(encode_end);
        total_times.push(total_end);
        output_size = bytes.len();
    }

    TimingResult {
        total_avg: avg_duration(&total_times),
        decode: avg_duration(&decode_times),
        process: avg_duration(&process_times),
        encode: avg_duration(&encode_times),
        output_size,
    }
}

/// Method 3: Convert to DynamicImage then use resize(Triangle)
fn benchmark_dynamic_image_resize(path: &Path, target_width: u32) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut process_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;

    for _ in 0..RUNS {
        let start = Instant::now();

        // Decode HEIC
        let path_str = path.to_string_lossy();
        let ctx = HeifContext::read_from_file(&path_str).unwrap();
        let handle = ctx.primary_image_handle().unwrap();
        let decode_end = start.elapsed();

        let lib_heif = LibHeif::new();
        let image = lib_heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None).unwrap();

        // Convert to DynamicImage
        let process_start = Instant::now();
        let (width, height, data) = get_rgba_from_heif(&image);
        let rgba_image = image::RgbaImage::from_raw(width, height, data).unwrap();
        let dynamic_image = image::DynamicImage::ImageRgba8(rgba_image);

        // Use resize(Triangle)
        let ratio = dynamic_image.height() as f64 / dynamic_image.width() as f64;
        let target_height = (target_width as f64 * ratio) as u32;
        let resized = dynamic_image.resize(target_width, target_height, image::imageops::FilterType::Triangle);
        let rgb_image = resized.to_rgb8();
        let process_end = process_start.elapsed();

        // Encode as JPEG
        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut bytes, 80);
        encoder.encode_image(&rgb_image).unwrap();
        let encode_end = encode_start.elapsed();

        let total_end = start.elapsed();

        decode_times.push(decode_end);
        process_times.push(process_end);
        encode_times.push(encode_end);
        total_times.push(total_end);
        output_size = bytes.len();
    }

    TimingResult {
        total_avg: avg_duration(&total_times),
        decode: avg_duration(&decode_times),
        process: avg_duration(&process_times),
        encode: avg_duration(&encode_times),
        output_size,
    }
}

/// Helper to extract RGBA data from libheif image
fn get_rgba_from_heif(heif_image: &libheif_rs::Image) -> (u32, u32, Vec<u8>) {
    let planes = heif_image.planes();
    let interleaved = planes.interleaved.as_ref().unwrap();
    let width = interleaved.width;
    let height = interleaved.height;
    let stride = interleaved.stride;
    let data = &interleaved.data;

    if stride == width as usize * 4 {
        (width, height, data.to_vec())
    } else {
        let mut rgb_data = Vec::with_capacity(width as usize * height as usize * 4);
        let bytes_per_row = width as usize * 4;
        for row in 0..height as usize {
            let row_offset = row * stride;
            rgb_data.extend_from_slice(&data[row_offset..row_offset + bytes_per_row]);
        }
        (width, height, rgb_data)
    }
}

fn print_result(method: &str, result: &TimingResult) {
    println!(
        "  {:<28} total={:>7.2}ms  decode={:>6.2}ms  process={:>6.2}ms  encode={:>6.2}ms  {}KB",
        method,
        result.total_avg.as_secs_f64() * 1000.0,
        result.decode.as_secs_f64() * 1000.0,
        result.process.as_secs_f64() * 1000.0,
        result.encode.as_secs_f64() * 1000.0,
        result.output_size / 1024,
    );
}

fn avg_duration(times: &[Duration]) -> Duration {
    let sum: Duration = times.iter().sum();
    sum / times.len() as u32
}
