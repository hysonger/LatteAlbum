//! Benchmark: Compare DynamicImage::thumbnail vs resize methods
//!
//! Usage: cargo run --example benchmark_thumbnail [jpg_path]
//!
//! This example compares the performance of:
//! - DynamicImage::thumbnail() - fast integer algorithm
//! - DynamicImage::resize() with Triangle - good quality/speed balance
//! - DynamicImage::resize() with Lanczos3 - highest quality (current default)
//!

use image::{ImageDecoder, ImageReader};
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

#[derive(Debug, Clone, Copy)]
struct FilterBench {
    name: &'static str,
    filter: image::imageops::FilterType,
}

const FILTERS: &[FilterBench] = &[
    FilterBench { name: "Lanczos3", filter: image::imageops::FilterType::Lanczos3 },
    FilterBench { name: "Triangle", filter: image::imageops::FilterType::Triangle },
    FilterBench { name: "Thumbnail", filter: image::imageops::FilterType::Triangle },
];

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --example benchmark_thumbnail <jpg_path>");
        std::process::exit(1);
    });

    let path = Path::new(&path);
    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        std::process::exit(1);
    }

    // Get image dimensions
    let dims = get_image_dimensions(path);
    println!("=== Thumbnail vs Resize Benchmark ===");
    println!("Image: {} ({}x{})", path.display(), dims.0, dims.1);
    println!("Runs per test: {}", RUNS);
    println!();

    // Benchmark each method
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

        // Benchmark resize with Triangle (our optimized choice)
        let triangle_result = benchmark_resize(path, target, image::imageops::FilterType::Triangle);
        print_result("resize+Triangle", &triangle_result);

        // Benchmark resize with Lanczos3 (original)
        let lanczos_result = benchmark_resize(path, target, image::imageops::FilterType::Lanczos3);
        print_result("resize+Lanczos3", &lanczos_result);

        // Benchmark thumbnail method (if resizing)
        if target > 0 {
            let thumbnail_result = benchmark_thumbnail(path, target);
            print_result("thumbnail()", &thumbnail_result);

            // Calculate speedup
            let speedup = lanczos_result.total_avg.as_secs_f64() / thumbnail_result.total_avg.as_secs_f64();
            println!(
                "  thumbnail() vs Lanczos3: {:.1}x faster ({:.1}%)",
                speedup,
                (speedup - 1.0) * 100.0
            );
        }
        println!();
    }

    // Summary comparison
    println!("=== Summary: thumbnail() vs resize(Lanczos3) ===");
    println!("Method          Small   Medium  Large");
    println!("----------------------------------------");

    for method in ["thumbnail()", "resize+Triangle", "resize+Lanczos3"] {
        let mut times = Vec::new();
        for target in [300, 450, 900] {
            let result = if method == "thumbnail()" {
                benchmark_thumbnail(path, target)
            } else if method == "resize+Triangle" {
                benchmark_resize(path, target, image::imageops::FilterType::Triangle)
            } else {
                benchmark_resize(path, target, image::imageops::FilterType::Lanczos3)
            };
            times.push(result.total_avg.as_secs_f64() * 1000.0);
        }
        println!("{:<16} {:>6.0}ms {:>6.0}ms {:>6.0}ms", method, times[0], times[1], times[2]);
    }
}

fn get_image_dimensions(path: &Path) -> (u32, u32) {
    let reader = ImageReader::open(path).unwrap();
    let decoder = reader.into_decoder().unwrap();
    decoder.dimensions()
}

/// Benchmark using DynamicImage::resize() with specified filter
fn benchmark_resize(path: &Path, target_width: u32, filter: image::imageops::FilterType) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut process_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;

    for _ in 0..RUNS {
        let start = Instant::now();

        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let decode_end = start.elapsed();

        let process_start = Instant::now();
        let result_img = if target_width == 0 {
            img.to_rgb8()
        } else {
            let ratio = img.height() as f64 / img.width() as f64;
            let target_height = (target_width as f64 * ratio) as u32;
            img.resize(target_width, target_height, filter).to_rgb8()
        };
        let process_end = process_start.elapsed();

        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 80);
        encoder.encode_image(&result_img).unwrap();
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

/// Benchmark using DynamicImage::thumbnail() method
fn benchmark_thumbnail(path: &Path, target_width: u32) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut process_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;

    for _ in 0..RUNS {
        let start = Instant::now();

        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let decode_end = start.elapsed();

        let process_start = Instant::now();
        // thumbnail maintains aspect ratio and uses fast integer algorithm
        let thumb = img.thumbnail(target_width, target_width);
        let result_img = thumb.to_rgb8();
        let process_end = process_start.elapsed();

        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 80);
        encoder.encode_image(&result_img).unwrap();
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

fn print_result(method: &str, result: &TimingResult) {
    println!(
        "  {:<18} total={:>7.2}ms  decode={:>6.2}ms  process={:>6.2}ms  encode={:>6.2}ms  {}KB",
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
