//! Benchmark: HEIC/JPG to JPEG/WebP transcoding comparison
//!
//! Usage: cargo run --example benchmark_format_transcode [heic_path] [jpg_path]
//!
//! This example benchmarks:
//! - HEIC to JPEG/WebP transcoding
//! - JPG to JPEG/WebP transcoding
//! - Compare output sizes and quality
//!
//! Tests both thumbnail sizes (300, 450, 900px) and full-size output.

use image::{codecs::jpeg::JpegEncoder, ImageDecoder, ImageReader};
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use std::path::Path;
use std::time::{Duration, Instant};
use webp;

const TARGET_SIZES: &[u32] = &[300, 450, 900, 0]; // small, medium, large, full
const RUNS: usize = 5;
const QUALITY: f32 = 0.8; // 80% quality for both JPEG and WebP

#[derive(Debug, Clone)]
struct TimingResult {
    total_avg: Duration,
    total_min: Duration,
    total_max: Duration,
    decode: Duration,
    process: Duration,
    encode: Duration,
    output_size: usize,
}

fn main() {
    let heic_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --example benchmark_format_transcode <heic_path> <jpg_path>");
        std::process::exit(1);
    });
    let jpg_path = std::env::args().nth(2).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --example benchmark_format_transcode <heic_path> <jpg_path>");
        std::process::exit(1);
    });

    let heic_path = Path::new(&heic_path);
    let jpg_path = Path::new(&jpg_path);

    if !heic_path.exists() {
        eprintln!("HEIC file not found: {}", heic_path.display());
        std::process::exit(1);
    }
    if !jpg_path.exists() {
        eprintln!("JPG file not found: {}", jpg_path.display());
        std::process::exit(1);
    }

    // Get image dimensions
    let heic_dim = get_heic_dimensions(heic_path);
    let jpg_dim = get_jpg_dimensions(jpg_path);

    println!("=== Format Transcode Benchmark ===");
    println!("HEIC: {} ({}x{})", heic_path.display(), heic_dim.0, heic_dim.1);
    println!("JPG: {} ({}x{})", jpg_path.display(), jpg_dim.0, jpg_dim.1);
    println!("Quality: {}%", (QUALITY * 100.0) as u8);
    println!("Runs per test: {}", RUNS);
    println!();

    // Summary table for each target size
    for &target in TARGET_SIZES {
        let target_name = match target {
            0 => "full",
            300 => "small",
            450 => "medium",
            900 => "large",
            _ => "custom",
        };
        println!("=== Target: {} ({}px) ===", target_name, if target == 0 { "original".to_string() } else { target.to_string() });

        // Benchmark all combinations
        let heic_to_jpg = benchmark_heic_to_jpg(heic_path, target);
        let heic_to_webp = benchmark_heic_to_webp(heic_path, target);
        let jpg_to_jpg = benchmark_jpg_to_jpg(jpg_path, target);
        let jpg_to_webp = benchmark_jpg_to_webp(jpg_path, target);

        // Print comparison table
        println!("Format       Total(ms)   Decode   Process   Encode    Size");
        println!("-------------------------------------------------------------");
        print_row("HEIC→JPEG", &heic_to_jpg);
        print_row("HEIC→WebP", &heic_to_webp);
        print_row("JPG→JPEG", &jpg_to_jpg);
        print_row("JPG→WebP", &jpg_to_webp);
        println!();

        // Size comparison
        println!("[Size Comparison at {}px]", if target == 0 { "original".to_string() } else { target.to_string() });
        let heic_jpg_size = heic_to_jpg.output_size;
        let heic_webp_size = heic_to_webp.output_size;
        let heic_ratio = heic_webp_size as f64 / heic_jpg_size as f64 * 100.0;
        println!("  HEIC: JPEG={}KB, WebP={}KB (WebP is {:.1}% of JPEG)", heic_jpg_size / 1024, heic_webp_size / 1024, heic_ratio);

        let jpg_jpg_size = jpg_to_jpg.output_size;
        let jpg_webp_size = jpg_to_webp.output_size;
        let jpg_ratio = jpg_webp_size as f64 / jpg_jpg_size as f64 * 100.0;
        println!("  JPG:  JPEG={}KB, WebP={}KB (WebP is {:.1}% of JPEG)", jpg_jpg_size / 1024, jpg_webp_size / 1024, jpg_ratio);
        println!();
    }

    // Performance summary
    println!("=== Performance Summary (Total Time) ===");
    println!("Target       HEIC→JPEG  HEIC→WebP  JPG→JPEG  JPG→WebP");
    println!("---------------------------------------------------------");
    for &target in TARGET_SIZES {
        let target_name = match target {
            0 => "full  ",
            300 => "small ",
            450 => "medium",
            900 => "large ",
            _ => "custom",
        };

        let heic_jpg = benchmark_heic_to_jpg(heic_path, target).total_avg.as_secs_f64() * 1000.0;
        let heic_webp = benchmark_heic_to_webp(heic_path, target).total_avg.as_secs_f64() * 1000.0;
        let jpg_jpg = benchmark_jpg_to_jpg(jpg_path, target).total_avg.as_secs_f64() * 1000.0;
        let jpg_webp = benchmark_jpg_to_webp(jpg_path, target).total_avg.as_secs_f64() * 1000.0;

        println!("{:9} {:>8.1}ms  {:>8.1}ms  {:>8.1}ms  {:>8.1}ms",
                 target_name, heic_jpg, heic_webp, jpg_jpg, jpg_webp);
    }

    // Size summary
    println!();
    println!("=== Size Summary (KB) ===");
    println!("Target       HEIC→JPEG  HEIC→WebP  JPG→JPEG  JPG→WebP");
    println!("---------------------------------------------------------");
    for &target in TARGET_SIZES {
        let target_name = match target {
            0 => "full  ",
            300 => "small ",
            450 => "medium",
            900 => "large ",
            _ => "custom",
        };

        let heic_jpg = benchmark_heic_to_jpg(heic_path, target).output_size / 1024;
        let heic_webp = benchmark_heic_to_webp(heic_path, target).output_size / 1024;
        let jpg_jpg = benchmark_jpg_to_jpg(jpg_path, target).output_size / 1024;
        let jpg_webp = benchmark_jpg_to_webp(jpg_path, target).output_size / 1024;

        println!("{:9} {:>9}KB  {:>9}KB  {:>9}KB  {:>9}KB",
                 target_name, heic_jpg, heic_webp, jpg_jpg, jpg_webp);
    }
}

// ==================== HEIC Tests ====================

fn benchmark_heic_to_jpg(path: &Path, target_width: u32) -> TimingResult {
    benchmark_heic_conversion(path, target_width, EncodeFormat::Jpeg)
}

fn benchmark_heic_to_webp(path: &Path, target_width: u32) -> TimingResult {
    benchmark_heic_conversion(path, target_width, EncodeFormat::WebP)
}

enum EncodeFormat {
    Jpeg,
    WebP,
}

fn benchmark_heic_conversion(path: &Path, target_width: u32, format: EncodeFormat) -> TimingResult {
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
        let image = lib_heif.decode(
            &handle,
            ColorSpace::Rgb(RgbChroma::Rgba),
            None,
        ).unwrap();

        // Scale using libheif's built-in scaling
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

        // Convert to RGB
        let (width, height, data) = get_rgba_from_heif(&scaled);
        let rgba_image = image::RgbaImage::from_raw(width, height, data).unwrap();
        let rgb_image = image::DynamicImage::ImageRgba8(rgba_image).to_rgb8();

        // Encode
        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        match format {
            EncodeFormat::Jpeg => {
                let mut encoder = JpegEncoder::new_with_quality(&mut bytes, (QUALITY * 100.0) as u8);
                encoder.encode_image(&rgb_image).unwrap();
            }
            EncodeFormat::WebP => {
                // Use webp crate for lossy WebP encoding with quality parameter
                // Convert ImageBuffer to DynamicImage for webp encoder
                let dynamic_img = image::DynamicImage::ImageRgb8(rgb_image);
                let encoder = webp::Encoder::from_image(&dynamic_img).unwrap();
                let webp_data = encoder.encode((QUALITY * 100.0) as f32);
                bytes.extend_from_slice(&webp_data);
            }
        }
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
        total_min: min_duration(&total_times),
        total_max: max_duration(&total_times),
        decode: avg_duration(&decode_times),
        process: avg_duration(&process_times),
        encode: avg_duration(&encode_times),
        output_size,
    }
}

// ==================== JPG Tests ====================

fn benchmark_jpg_to_jpg(path: &Path, target_width: u32) -> TimingResult {
    benchmark_jpg_conversion(path, target_width, EncodeFormat::Jpeg)
}

fn benchmark_jpg_to_webp(path: &Path, target_width: u32) -> TimingResult {
    benchmark_jpg_conversion(path, target_width, EncodeFormat::WebP)
}

fn benchmark_jpg_conversion(path: &Path, target_width: u32, format: EncodeFormat) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut process_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;

    for _ in 0..RUNS {
        let start = Instant::now();

        // Decode JPG
        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let decode_end = start.elapsed();

        // Resize using thumbnail() method (fast integer algorithm)
        let process_start = Instant::now();
        let result_img = if target_width == 0 {
            img.to_rgb8()
        } else {
            let thumb = img.thumbnail(target_width, target_width);
            thumb.to_rgb8()
        };
        let process_end = process_start.elapsed();

        // Encode
        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        match format {
            EncodeFormat::Jpeg => {
                let mut encoder = JpegEncoder::new_with_quality(&mut bytes, (QUALITY * 100.0) as u8);
                encoder.encode_image(&result_img).unwrap();
            }
            EncodeFormat::WebP => {
                // Use webp crate for lossy WebP encoding with quality parameter
                // Convert ImageBuffer to DynamicImage for webp encoder
                let dynamic_img = image::DynamicImage::ImageRgb8(result_img);
                let encoder = webp::Encoder::from_image(&dynamic_img).unwrap();
                let webp_data = encoder.encode((QUALITY * 100.0) as f32);
                bytes.extend_from_slice(&webp_data);
            }
        }
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
        total_min: min_duration(&total_times),
        total_max: max_duration(&total_times),
        decode: avg_duration(&decode_times),
        process: avg_duration(&process_times),
        encode: avg_duration(&encode_times),
        output_size,
    }
}

// ==================== Helper Functions ====================

fn get_heic_dimensions(path: &Path) -> (u32, u32) {
    let path_str = path.to_string_lossy();
    let ctx = HeifContext::read_from_file(&path_str).unwrap();
    let handle = ctx.primary_image_handle().unwrap();
    (handle.width(), handle.height())
}

fn get_jpg_dimensions(path: &Path) -> (u32, u32) {
    let reader = ImageReader::open(path).unwrap();
    let decoder = reader.into_decoder().unwrap();
    decoder.dimensions()
}

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

fn print_row(label: &str, result: &TimingResult) {
    println!(
        "{:<12} {:>7.2}ms   {:>6.2}ms   {:>6.2}ms   {:>6.2}ms   {}KB",
        label,
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

fn min_duration(times: &[Duration]) -> Duration {
    *times.iter().min().unwrap()
}

fn max_duration(times: &[Duration]) -> Duration {
    *times.iter().max().unwrap()
}
