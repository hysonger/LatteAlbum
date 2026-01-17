//! Benchmark: HEIC vs JPG transcoding performance comparison
//!
//! Usage: cargo run --example benchmark_transcode [photos_dir]
//!
//! This example benchmarks HEIC and JPG thumbnail generation, measuring
//! decode, resize, and encode times separately to identify bottlenecks.

use image::{ImageDecoder, ImageReader};
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const TARGET_SIZES: &[u32] = &[300, 450, 900, 0]; // small, medium, large, full
const RUNS: usize = 5;

#[derive(Debug, Clone)]
struct TimingResult {
    total_avg: Duration,
    total_min: Duration,
    total_max: Duration,
    decode: Duration,
    resize: Duration,
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
    FilterBench { name: "CatmullRom", filter: image::imageops::FilterType::CatmullRom },
    FilterBench { name: "Gaussian", filter: image::imageops::FilterType::Gaussian },
    FilterBench { name: "Triangle", filter: image::imageops::FilterType::Triangle },
    FilterBench { name: "Nearest", filter: image::imageops::FilterType::Nearest },
];

fn main() {
    let photos_dir = std::env::args().nth(1).unwrap_or_else(|| {
        println!("Usage: cargo run --example benchmark_transcode [photos_dir]");
        std::process::exit(1);
    });

    let photos_path = Path::new(&photos_dir);
    if !photos_path.exists() {
        eprintln!("Directory not found: {}", photos_path.display());
        std::process::exit(1);
    }

    println!("=== HEIC vs JPG Transcoding Benchmark ===");
    println!("Photos directory: {}", photos_path.display());
    println!("Runs per test: {}", RUNS);
    println!();

    // Find paired files
    let pairs = find_paired_files(photos_path);
    if pairs.is_empty() {
        eprintln!("No paired HEIC/JPG files found in {}", photos_path.display());
        eprintln!("Looking for files with same name but .heic/.jpg/.jpeg extensions");
        std::process::exit(1);
    }

    println!("Found {} paired file sets\n", pairs.len());

    for (heic_path, jpg_path) in &pairs {
        benchmark_pair(heic_path, jpg_path);
        println!();
    }

    // Algorithm comparison summary
    if !pairs.is_empty() {
        println!("=== Algorithm Comparison Summary ===");
        println!("Testing all filters on first pair at medium size (450px)...\n");
        if let Some((heic_path, jpg_path)) = pairs.first() {
            benchmark_algorithms(jpg_path, 450);
        }
    }
}

fn find_paired_files(photos_dir: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = Vec::new();

    // Collect all HEIC files
    let heic_files: Vec<PathBuf> = photos_dir
        .read_dir()
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let ext = path.extension()?.to_str()?;
            if ext.eq_ignore_ascii_case("heic") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    for heic_path in heic_files {
        let stem = heic_path.file_stem().unwrap().to_str().unwrap();
        let parent = heic_path.parent().unwrap();

        // Look for matching JPG
        let jpg_path = parent.join(format!("{}.jpg", stem));
        let jpeg_path = parent.join(format!("{}.jpeg", stem));

        let jpg = if jpg_path.exists() {
            Some(jpg_path)
        } else if jpeg_path.exists() {
            Some(jpeg_path)
        } else {
            None
        };

        if let Some(jpg) = jpg {
            pairs.push((heic_path, jpg));
        }
    }

    pairs
}

fn benchmark_pair(heic_path: &Path, jpg_path: &Path) {
    let file_name = heic_path.file_stem().unwrap().to_str().unwrap();
    println!("=== Benchmark: {} ===", file_name);

    // Get image dimensions
    let heic_dim = get_heic_dimensions(heic_path);
    let jpg_dim = get_jpg_dimensions(jpg_path);
    println!("HEIC: {}x{}, JPG: {}x{}", heic_dim.0, heic_dim.1, jpg_dim.0, jpg_dim.1);
    println!();

    for &target in TARGET_SIZES {
        let target_name = match target {
            0 => "full",
            300 => "small",
            450 => "medium",
            900 => "large",
            _ => "custom",
        };

        // HEIC benchmark
        let heic_result = benchmark_heic(heic_path, target, RUNS);
        print_result("HEIC", target_name, &heic_result, target);

        // JPG benchmark with Lanczos3 (current)
        let jpg_result = benchmark_jpg(jpg_path, target, image::imageops::FilterType::Lanczos3, RUNS);
        print_result("JPG", target_name, &jpg_result, target);

        // Speed comparison
        let speedup = jpg_result.total_avg.as_secs_f64() / heic_result.total_avg.as_secs_f64();
        println!(
            "  [{}] HEIC vs JPG: {:.2}x {}",
            target_name,
            speedup,
            if speedup > 1.0 {
                format!("(JPG slower by {:.0}%)", (speedup - 1.0) * 100.0)
            } else {
                format!("(HEIC slower by {:.0}%)", (1.0 / speedup - 1.0) * 100.0)
            }
        );
        println!();
    }
}

fn benchmark_heic(path: &Path, target_width: u32, runs: usize) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut resize_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;

    for _ in 0..runs {
        let start = Instant::now();

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

        let resize_start = Instant::now();
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
        let resize_end = resize_start.elapsed();

        let planes = scaled.planes();
        let interleaved = planes.interleaved.as_ref().unwrap();
        let width = interleaved.width;
        let height = interleaved.height;
        let stride = interleaved.stride;
        let data = &interleaved.data;

        let rgba_image = if stride == width as usize * 4 {
            image::RgbaImage::from_raw(width, height, data.to_vec()).unwrap()
        } else {
            let mut rgb_data = Vec::with_capacity(width as usize * height as usize * 4);
            let bytes_per_row = width as usize * 4;
            for row in 0..height as usize {
                let row_offset = row * stride;
                rgb_data.extend_from_slice(&data[row_offset..row_offset + bytes_per_row]);
            }
            image::RgbaImage::from_raw(width, height, rgb_data).unwrap()
        };

        let rgb_image = image::DynamicImage::ImageRgba8(rgba_image).to_rgb8();

        let encode_start = Instant::now();
        let mut jpeg_bytes = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_bytes, 80);
        encoder.encode_image(&rgb_image).unwrap();
        let encode_end = encode_start.elapsed();

        let total_end = start.elapsed();

        decode_times.push(decode_end);
        resize_times.push(resize_end);
        encode_times.push(encode_end);
        total_times.push(total_end);
        output_size = jpeg_bytes.len();
    }

    TimingResult {
        total_avg: avg_duration(&total_times),
        total_min: min_duration(&total_times),
        total_max: max_duration(&total_times),
        decode: avg_duration(&decode_times),
        resize: avg_duration(&resize_times),
        encode: avg_duration(&encode_times),
        output_size,
    }
}

fn benchmark_jpg(path: &Path, target_width: u32, filter: image::imageops::FilterType, runs: usize) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut resize_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;

    for _ in 0..runs {
        let start = Instant::now();

        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let decode_end = start.elapsed();


        let resize_start = Instant::now();
        let result_img = if target_width == 0 {
            img.to_rgb8()
        } else {
            let ratio = img.height() as f64 / img.width() as f64;
            let target_height = (target_width as f64 * ratio) as u32;
            img.resize(target_width, target_height, filter).to_rgb8()
        };
        let resize_end = resize_start.elapsed();

        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 80);
        encoder.encode_image(&result_img).unwrap();
        let encode_end = encode_start.elapsed();

        let total_end = start.elapsed();

        decode_times.push(decode_end);
        resize_times.push(resize_end);
        encode_times.push(encode_end);
        total_times.push(total_end);
        output_size = bytes.len();
    }

    TimingResult {
        total_avg: avg_duration(&total_times),
        total_min: min_duration(&total_times),
        total_max: max_duration(&total_times),
        decode: avg_duration(&decode_times),
        resize: avg_duration(&resize_times),
        encode: avg_duration(&encode_times),
        output_size,
    }
}

fn benchmark_algorithms(path: &Path, target_width: u32) {
    println!("Filter       Avg(ms)   Decode   Resize   Encode   Output");
    println!("--------------------------------------------------------");

    for filter_bench in FILTERS {
        let result = benchmark_jpg(path, target_width, filter_bench.filter, RUNS);
        println!(
            "{:<12} {:>7.2}   {:>6.2}   {:>6.2}   {:>6.2}   {} KB",
            filter_bench.name,
            result.total_avg.as_secs_f64() * 1000.0,
            result.decode.as_secs_f64() * 1000.0,
            result.resize.as_secs_f64() * 1000.0,
            result.encode.as_secs_f64() * 1000.0,
            result.output_size / 1024,
        );
    }
}

fn print_result(format: &str, target: &str, result: &TimingResult, target_px: u32) {
    let target_display = if target_px == 0 { "0px" } else { &format!("{}px", target_px) };
    println!(
        "Format: {}, Target: {} ({})",
        format,
        target,
        target_display
    );
    println!(
        "  Total:    avg={:.2}ms (min={:.2}ms, max={:.2}ms)",
        result.total_avg.as_secs_f64() * 1000.0,
        result.total_min.as_secs_f64() * 1000.0,
        result.total_max.as_secs_f64() * 1000.0,
    );
    println!(
        "  Decode:   {:.2}ms",
        result.decode.as_secs_f64() * 1000.0,
    );
    println!(
        "  Resize:   {:.2}ms",
        result.resize.as_secs_f64() * 1000.0,
    );
    println!(
        "  Encode:   {:.2}ms",
        result.encode.as_secs_f64() * 1000.0,
    );
    println!("  Output:   {} KB", result.output_size / 1024);
}

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
