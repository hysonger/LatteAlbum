//! Benchmark: Image crate + libheif-rs integration thumbnail performance
//!
//! Usage:
//!   cargo run --example bench_thumbnail_image_heif_jpg <image_path>
//!   cargo run --example bench_thumbnail_image_heif_jpg <heic_path> <jpg_path>
//!
//! This example benchmarks thumbnail generation comparing:
//! 1. image crate + libheif-rs integration (thumbnail() method)
//! 2. image crate + libheif-rs integration (resize Triangle)
//! 3. Pure libheif-rs (scale() + Triangle via image crate)
//! 4. Pure libheif-rs (scale() + thumbnail() via image crate)
//!
//! Compares HEIF and JPG files at sizes: small(300px), medium(450px), large(900px), full(original)

use image::codecs::jpeg::JpegEncoder;
use image::{ImageReader, ImageBuffer};
use libheif_rs::integration::image::register_all_decoding_hooks;
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use rayon::prelude::*;
use std::path::Path;
use std::time::{Duration, Instant};

const TARGET_SIZES: &[u32] = &[300, 450, 900, 0]; // small, medium, large, full
const RUNS: usize = 5;
const JPEG_QUALITY: u8 = 80;

#[derive(Debug, Clone)]
struct TimingResult {
    total_avg: Duration,
    decode: Duration,
    process: Duration,
    encode: Duration,
    output_size: usize,
    output_width: u32,
    output_height: u32,
}

#[derive(Debug)]
struct BenchmarkFile {
    path: std::path::PathBuf,
    ext: String,
    is_heif: bool,
    width: u32,
    height: u32,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <image_path>", args[0]);
        eprintln!("   or: {} <heic_path> <jpg_path>  (compare HEIF vs JPG)", args[0]);
        eprintln!("");
        eprintln!("Supports: HEIC, HEIF, AVIF, JPG, PNG, TIFF, WebP, etc.");
        std::process::exit(1);
    }

    // Register libheif-rs decoding hooks with image crate
    println!("Registering libheif-rs decoding hooks with image crate...");
    register_all_decoding_hooks();
    println!("Hooks registered successfully.\n");

    let files: Vec<BenchmarkFile> = if args.len() >= 3 {
        // Compare mode: HEIC vs JPG
        let heic_path = Path::new(&args[1]);
        let jpg_path = Path::new(&args[2]);

        if !heic_path.exists() {
            eprintln!("File not found: {}", heic_path.display());
            std::process::exit(1);
        }
        if !jpg_path.exists() {
            eprintln!("File not found: {}", jpg_path.display());
            std::process::exit(1);
        }

        vec![
            create_benchmark_file(heic_path),
            create_benchmark_file(jpg_path),
        ]
    } else {
        // Single file mode
        let path = Path::new(&args[1]);
        if !path.exists() {
            eprintln!("File not found: {}", path.display());
            std::process::exit(1);
        }
        vec![create_benchmark_file(path)]
    };

    // Run benchmarks
    for file in &files {
        run_benchmark(file);
        println!();
    }

    // Compare mode: show comparison table
    if files.len() == 2 {
        print_comparison(&files[0], &files[1]);
    }
}

fn create_benchmark_file(path: &Path) -> BenchmarkFile {
    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let is_heif = matches!(ext.as_str(), "heic" | "heif" | "avif");

    // Get dimensions
    let dims = get_image_dimensions(path);

    BenchmarkFile {
        path: path.to_path_buf(),
        ext,
        is_heif,
        width: dims.0,
        height: dims.1,
    }
}

fn run_benchmark(file: &BenchmarkFile) {
    let format_name = if file.is_heif { "HEIF" } else if file.ext == "jpg" || file.ext == "jpeg" { "JPG" } else { &file.ext };

    println!("=== {} Thumbnail Benchmark (image crate + libheif-rs) ===", format_name.to_uppercase());
    println!("File: {}", file.path.display());
    println!("Dimensions: {} x {}", file.width, file.height);
    println!("Runs per test: {} (first run warmup, avg of last {})\n", RUNS, RUNS - 1);

    // Benchmark each target size
    for target in TARGET_SIZES {
        let target = *target;
        let target_name = match target {
            0 => "full (original)",
            300 => "small",
            450 => "medium",
            900 => "large",
            _ => "custom",
        };

        println!("  {} ({})", target_name, if target > 0 { format!("{}px", target) } else { "original".to_string() });

        // Method 1: image crate + libheif (thumbnail)
        let result1 = benchmark_image_thumbnail(file, target);
        print_result("image+libheif thumbnail()", &result1);

        // Method 2: image crate + libheif (resize Triangle)
        if target > 0 {
            let result2 = benchmark_image_resize_triangle(file, target);
            print_result("image+libheif resize(Triangle)", &result2);
        }

        // For HEIF files only: pure libheif methods
        if file.is_heif {
            // Method 3: pure libheif (scale) + thumbnail
            let result3 = benchmark_pure_libheif_thumbnail(file, target);
            print_result("pure libheif scale+thumbnail", &result3);

            // Method 4: pure libheif (scale) + resize Triangle
            if target > 0 {
                let result4 = benchmark_pure_libheif_triangle(file, target);
                print_result("pure libheif scale+Triangle", &result4);

                // Method 5: pure libheif native scale (no image crate resize/thumbnail)
                let result5 = benchmark_pure_libheif_native(file, target);
                print_result("pure libheif native", &result5);
            }
        }

        println!();
    }
}

/// Get image dimensions using image crate
fn get_image_dimensions(path: &Path) -> (u32, u32) {
    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    // For HEIF/AVIF, need to decode
    if matches!(ext.as_str(), "heic" | "heif" | "avif") {
        let reader = ImageReader::open(path).unwrap();
        let image = reader.decode().unwrap();
        (image.width(), image.height())
    } else {
        // For common formats, decode and get dimensions
        let img = ImageReader::open(path).unwrap().decode().unwrap();
        (img.width(), img.height())
    }
}

/// Method 1: image crate + libheif-rs integration with thumbnail()
fn benchmark_image_thumbnail(file: &BenchmarkFile, target_width: u32) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut process_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;
    let mut output_width = 0;
    let mut output_height = 0;

    for run in 0..RUNS {
        let start = Instant::now();

        // Decode using image crate with registered hooks
        let decode_start = Instant::now();
        let reader = ImageReader::open(&file.path).unwrap();
        let image = reader.decode().unwrap();
        let decode_end = decode_start.elapsed();

        // Scale using thumbnail()
        let process_start = Instant::now();
        let processed_image = if target_width == 0 {
            image.to_rgb8()
        } else {
            let thumb = image.thumbnail(target_width, target_width);
            thumb.to_rgb8()
        };

        // Calculate output dimensions
        if target_width == 0 {
            output_width = image.width();
            output_height = image.height();
        } else {
            let ratio = image.height() as f64 / image.width() as f64;
            output_width = target_width;
            output_height = (target_width as f64 * ratio) as u32;
        }
        let process_end = process_start.elapsed();

        // Encode as JPEG
        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        {
            let mut encoder = JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY);
            encoder.encode_image(&processed_image).unwrap();
        }
        let encode_end = encode_start.elapsed();

        let total_end = start.elapsed();

        if run > 0 {
            decode_times.push(decode_end);
            process_times.push(process_end);
            encode_times.push(encode_end);
            total_times.push(total_end);
            output_size = bytes.len();
        } else {
            output_size = bytes.len();
        }
    }

    TimingResult {
        total_avg: avg_duration(&total_times),
        decode: avg_duration(&decode_times),
        process: avg_duration(&process_times),
        encode: avg_duration(&encode_times),
        output_size,
        output_width,
        output_height,
    }
}

/// Method 2: image crate + libheif-rs integration with resize(Triangle)
fn benchmark_image_resize_triangle(file: &BenchmarkFile, target_width: u32) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut process_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;
    let mut output_width = 0;
    let mut output_height = 0;

    for run in 0..RUNS {
        let start = Instant::now();

        // Decode using image crate with registered hooks
        let decode_start = Instant::now();
        let reader = ImageReader::open(&file.path).unwrap();
        let image = reader.decode().unwrap();
        let decode_end = decode_start.elapsed();

        // Scale using resize(Triangle)
        let process_start = Instant::now();
        let ratio = image.height() as f64 / image.width() as f64;
        let target_height = (target_width as f64 * ratio) as u32;
        let resized = image.resize(target_width, target_height, image::imageops::FilterType::Triangle);
        let processed_image = resized.to_rgb8();

        output_width = target_width;
        output_height = target_height;
        let process_end = process_start.elapsed();

        // Encode as JPEG
        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        {
            let mut encoder = JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY);
            encoder.encode_image(&processed_image).unwrap();
        }
        let encode_end = encode_start.elapsed();

        let total_end = start.elapsed();

        if run > 0 {
            decode_times.push(decode_end);
            process_times.push(process_end);
            encode_times.push(encode_end);
            total_times.push(total_end);
            output_size = bytes.len();
        } else {
            output_size = bytes.len();
        }
    }

    TimingResult {
        total_avg: avg_duration(&total_times),
        decode: avg_duration(&decode_times),
        process: avg_duration(&process_times),
        encode: avg_duration(&encode_times),
        output_size,
        output_width,
        output_height,
    }
}

/// Method 3: Pure libheif-rs with scale() + image thumbnail()
fn benchmark_pure_libheif_thumbnail(file: &BenchmarkFile, target_width: u32) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut process_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;
    let mut output_width = 0;
    let mut output_height = 0;

    for run in 0..RUNS {
        let start = Instant::now();

        // Decode using pure libheif-rs
        let decode_start = Instant::now();
        let path_str = file.path.to_string_lossy();
        let ctx = HeifContext::read_from_file(&path_str).unwrap();
        let handle = ctx.primary_image_handle().unwrap();
        let lib_heif = LibHeif::new();
        let heif_image = lib_heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None).unwrap();
        let decode_end = decode_start.elapsed();

        // Scale using libheif scale()
        let process_start = Instant::now();
        let ratio = heif_image.height() as f64 / heif_image.width() as f64;

        let (width, height, rgb_data) = if target_width == 0 {
            // Full size: skip scale, directly extract and convert RGBA -> RGB
            let (w, h, rgba_data) = get_rgba_from_heif(&heif_image);
            let rgb = rgba_to_rgb(&rgba_data);
            (w, h, rgb)
        } else {
            // Thumbnail: scale first, then extract
            let target_h = (target_width as f64 * ratio) as u32;
            let scaled_heif = if heif_image.width() > target_width || heif_image.height() > target_h {
                heif_image.scale(target_width, target_h, None).unwrap()
            } else {
                heif_image.scale(heif_image.width(), heif_image.height(), None).unwrap()
            };
            let (w, h, rgba_data) = get_rgba_from_heif(&scaled_heif);

            // Use thumbnail() method
            let rgba_image = ImageBuffer::from_raw(w, h, rgba_data).unwrap();
            let dynamic_image = image::DynamicImage::ImageRgba8(rgba_image);
            let thumb = dynamic_image.thumbnail(target_width, target_width);
            let rgb = thumb.to_rgb8();
            (thumb.width(), thumb.height(), rgb.into_raw())
        };

        output_width = width;
        output_height = height;
        let process_end = process_start.elapsed();

        // Encode as JPEG
        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        {
            let rgb_image: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, rgb_data).unwrap();
            let mut encoder = JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY);
            encoder.encode_image(&rgb_image).unwrap();
        }
        let encode_end = encode_start.elapsed();

        let total_end = start.elapsed();

        if run > 0 {
            decode_times.push(decode_end);
            process_times.push(process_end);
            encode_times.push(encode_end);
            total_times.push(total_end);
            output_size = bytes.len();
        } else {
            output_size = bytes.len();
        }
    }

    TimingResult {
        total_avg: avg_duration(&total_times),
        decode: avg_duration(&decode_times),
        process: avg_duration(&process_times),
        encode: avg_duration(&encode_times),
        output_size,
        output_width,
        output_height,
    }
}

/// Method 4: Pure libheif-rs with scale() + image resize(Triangle)
fn benchmark_pure_libheif_triangle(file: &BenchmarkFile, target_width: u32) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut process_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;
    let mut output_width = 0;
    let mut output_height = 0;

    for run in 0..RUNS {
        let start = Instant::now();

        // Decode using pure libheif-rs
        let decode_start = Instant::now();
        let path_str = file.path.to_string_lossy();
        let ctx = HeifContext::read_from_file(&path_str).unwrap();
        let handle = ctx.primary_image_handle().unwrap();
        let lib_heif = LibHeif::new();
        let heif_image = lib_heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None).unwrap();
        let decode_end = decode_start.elapsed();

        // Scale using libheif scale()
        let process_start = Instant::now();
        let ratio = heif_image.height() as f64 / heif_image.width() as f64;

        let (width, height, rgb_data) = if target_width == 0 {
            // Full size: skip scale, directly extract and convert RGBA -> RGB
            let (w, h, rgba_data) = get_rgba_from_heif(&heif_image);
            let rgb = rgba_to_rgb(&rgba_data);
            (w, h, rgb)
        } else {
            // Thumbnail: scale first, then extract
            let target_h = (target_width as f64 * ratio) as u32;
            let scaled_heif = if heif_image.width() > target_width || heif_image.height() > target_h {
                heif_image.scale(target_width, target_h, None).unwrap()
            } else {
                heif_image.scale(heif_image.width(), heif_image.height(), None).unwrap()
            };
            let (w, h, rgba_data) = get_rgba_from_heif(&scaled_heif);

            // Use Triangle resize
            let rgba_image = ImageBuffer::from_raw(w, h, rgba_data).unwrap();
            let dynamic_image = image::DynamicImage::ImageRgba8(rgba_image);
            let resized = dynamic_image.resize(target_width, target_width, image::imageops::FilterType::Triangle);
            let rgb = resized.to_rgb8();
            (resized.width(), resized.height(), rgb.into_raw())
        };

        output_width = width;
        output_height = height;
        let process_end = process_start.elapsed();

        // Encode as JPEG
        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        {
            let rgb_image: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, rgb_data).unwrap();
            let mut encoder = JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY);
            encoder.encode_image(&rgb_image).unwrap();
        }
        let encode_end = encode_start.elapsed();

        let total_end = start.elapsed();

        if run > 0 {
            decode_times.push(decode_end);
            process_times.push(process_end);
            encode_times.push(encode_end);
            total_times.push(total_end);
            output_size = bytes.len();
        } else {
            output_size = bytes.len();
        }
    }

    TimingResult {
        total_avg: avg_duration(&total_times),
        decode: avg_duration(&decode_times),
        process: avg_duration(&process_times),
        encode: avg_duration(&encode_times),
        output_size,
        output_width,
        output_height,
    }
}

/// Method 5: Pure libheif-rs with native scale only (no image crate resize)
fn benchmark_pure_libheif_native(file: &BenchmarkFile, target_width: u32) -> TimingResult {
    let mut decode_times = Vec::new();
    let mut process_times = Vec::new();
    let mut encode_times = Vec::new();
    let mut total_times = Vec::new();
    let mut output_size = 0;
    let mut output_width = 0;
    let mut output_height = 0;

    for run in 0..RUNS {
        let start = Instant::now();

        // Decode using pure libheif-rs
        let decode_start = Instant::now();
        let path_str = file.path.to_string_lossy();
        let ctx = HeifContext::read_from_file(&path_str).unwrap();
        let handle = ctx.primary_image_handle().unwrap();
        let lib_heif = LibHeif::new();
        let heif_image = lib_heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None).unwrap();
        let decode_end = decode_start.elapsed();

        // Scale using libheif's built-in scale()
        let process_start = Instant::now();
        let ratio = heif_image.height() as f64 / heif_image.width() as f64;

        let (width, height, rgb_data) = if target_width == 0 {
            // Full size: skip scale, directly extract RGBA -> RGB
            let (w, h, rgba_data) = get_rgba_from_heif(&heif_image);
            let rgb = rgba_to_rgb(&rgba_data);
            (w, h, rgb)
        } else {
            // Thumbnail: use libheif native scale
            let target_h = (target_width as f64 * ratio) as u32;
            let scaled_heif = if heif_image.width() > target_width || heif_image.height() > target_h {
                heif_image.scale(target_width, target_h, None).unwrap()
            } else {
                heif_image.scale(heif_image.width(), heif_image.height(), None).unwrap()
            };

            // Extract and convert directly (no image crate resize/thumbnail)
            let (w, h, rgba_data) = get_rgba_from_heif(&scaled_heif);
            let rgb = rgba_to_rgb(&rgba_data);
            (w, h, rgb)
        };

        output_width = width;
        output_height = height;
        let process_end = process_start.elapsed();

        // Encode as JPEG
        let encode_start = Instant::now();
        let mut bytes = Vec::new();
        {
            let rgb_image: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, rgb_data).unwrap();
            let mut encoder = JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY);
            encoder.encode_image(&rgb_image).unwrap();
        }
        let encode_end = encode_start.elapsed();

        let total_end = start.elapsed();

        if run > 0 {
            decode_times.push(decode_end);
            process_times.push(process_end);
            encode_times.push(encode_end);
            total_times.push(total_end);
            output_size = bytes.len();
        } else {
            output_size = bytes.len();
        }
    }

    TimingResult {
        total_avg: avg_duration(&total_times),
        decode: avg_duration(&decode_times),
        process: avg_duration(&process_times),
        encode: avg_duration(&encode_times),
        output_size,
        output_width,
        output_height,
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
        let bytes_per_row = width as usize * 4;
        // Use rayon for parallel row processing
        let rows: Vec<_> = (0..height as usize).collect();
        let rgb_data: Vec<u8> = rows.par_iter()
            .flat_map(|&row| {
                let row_offset = row * stride;
                data[row_offset..row_offset + bytes_per_row].to_vec()
            })
            .collect();
        (width, height, rgb_data)
    }
}

/// Helper to convert RGBA data to RGB (strips alpha channel) - parallelized
fn rgba_to_rgb(rgba: &[u8]) -> Vec<u8> {
    // Use rayon for parallel RGBA -> RGB conversion
    rgba.par_chunks(4)
        .flat_map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
        .collect()
}

fn print_result(method: &str, result: &TimingResult) {
    let size_kb = result.output_size as f64 / 1024.0;
    let resolution = format!("{}x{}", result.output_width, result.output_height);

    println!(
        "    {:<28} total={:>7.2}ms  decode={:>6.2}ms  scale={:>6.2}ms  encode={:>6.2}ms  {} ({:.1}KB)",
        method,
        result.total_avg.as_secs_f64() * 1000.0,
        result.decode.as_secs_f64() * 1000.0,
        result.process.as_secs_f64() * 1000.0,
        result.encode.as_secs_f64() * 1000.0,
        resolution,
        size_kb,
    );
}

fn avg_duration(times: &[Duration]) -> Duration {
    if times.is_empty() {
        return Duration::ZERO;
    }
    let sum: Duration = times.iter().sum();
    sum / times.len() as u32
}

/// Print comparison table between two files
fn print_comparison(heif_file: &BenchmarkFile, jpg_file: &BenchmarkFile) {
    println!("=== Comparison: HEIF vs JPG ===");
    println!("HEIF: {} ({}x{})", heif_file.path.display(), heif_file.width, heif_file.height);
    println!("JPG:  {} ({}x{})\n", jpg_file.path.display(), jpg_file.width, jpg_file.height);

    println!("{:<12} {:>12} {:>12} {:>12}", "Size", "HEIF Total", "JPG Total", "HEIF/JPG");
    println!("{}", "-".repeat(52));

    for target in TARGET_SIZES {
        let heif_result = benchmark_image_thumbnail(heif_file, *target);
        let jpg_result = benchmark_image_thumbnail(jpg_file, *target);

        let size_name = match *target {
            0 => "full",
            300 => "small",
            450 => "medium",
            900 => "large",
            _ => "custom",
        };

        let ratio = if jpg_result.total_avg.as_secs_f64() > 0.0 {
            heif_result.total_avg.as_secs_f64() / jpg_result.total_avg.as_secs_f64()
        } else {
            0.0
        };

        println!(
            "{:<12} {:>10.2}ms {:>10.2}ms {:>11.2}x",
            size_name,
            heif_result.total_avg.as_secs_f64() * 1000.0,
            jpg_result.total_avg.as_secs_f64() * 1000.0,
            ratio,
        );
    }

    println!();
    println!("Note: HEIF files are typically smaller in file size but require");
    println!("      additional decoding overhead. JPG files decode faster but");
    println!("      may have larger file sizes for the same quality.");
}
