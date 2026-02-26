//! Video thumbnail generation test utility using ffmpeg-next library
//!
//! Usage: cargo run --features vendor-build --example video_thumbnail_with_rotation <input_video> [output.jpg] [rotation]
//!
//! Extracts a frame from video at specified offset and converts to JPEG.
//! Optional rotation parameter (in degrees): 90, -90, 180, 0, etc.

use ffmpeg_next::format::input;
use ffmpeg_next::media::Type;
use ffmpeg_next::codec::context::Context;
use ffmpeg_next::software::scaling::{Context as ScalingContext, Flags};
use ffmpeg_next::format::Pixel;
use ffmpeg_next::util::frame::video::Video;
use ffmpeg_next::codec::packet::side_data::Type as PacketSideDataType;
use std::env;
use std::path::PathBuf;

/// Get rotation angle from video stream's side_data (DisplayMatrix)
/// Note: Modern video files often store rotation in stream tags (rotate),
/// which requires accessing metadata through the format context.
fn get_rotation_angle(stream: &ffmpeg_next::Stream) -> Option<i32> {
    // Debug: list all side_data
    println!("[DEBUG] Checking side_data for rotation...");

    for side_data in stream.side_data() {
        let kind = side_data.kind();
        println!("[DEBUG] Found side_data: {:?}", kind);

        if kind == PacketSideDataType::DisplayMatrix {
            let data = side_data.data();
            println!("[DEBUG] DisplayMatrix data length: {} bytes", data.len());

            // DisplayMatrix is a 3x3 matrix of int32_t (9 x 4 = 36 bytes)
            // Format: 16.16 fixed-point representation
            // Layout: [a, b, u, c, d, v, x, y, w] representing a 3x3 matrix
            if data.len() >= 36 {
                // Read matrix as slice of i32
                let matrix: &[i32] = unsafe {
                    std::slice::from_raw_parts(
                        data.as_ptr() as *const i32,
                        9
                    )
                };

                // Convert from fixed-point (16.16) to floating-point
                let conv_fp = |x: i32| x as f64 / (1i32 << 16) as f64;

                // Debug: print the complete 3x3 matrix in fixed-point
                println!("[DEBUG] Display Matrix (fixed-point):");
                println!("[DEBUG]   [{:12}, {:12}, {:12}]", matrix[0], matrix[1], matrix[2]);
                println!("[DEBUG]   [{:12}, {:12}, {:12}]", matrix[3], matrix[4], matrix[5]);
                println!("[DEBUG]   [{:12}, {:12}, {:12}]", matrix[6], matrix[7], matrix[8]);

                // Debug: print converted floating-point values
                println!("[DEBUG] Display Matrix (floating-point):");
                println!("[DEBUG]   [{:12.6}, {:12.6}, {:12.6}]", conv_fp(matrix[0]), conv_fp(matrix[1]), conv_fp(matrix[2]));
                println!("[DEBUG]   [{:12.6}, {:12.6}, {:12.6}]", conv_fp(matrix[3]), conv_fp(matrix[4]), conv_fp(matrix[5]));
                println!("[DEBUG]   [{:12.6}, {:12.6}, {:12.6}]", conv_fp(matrix[6]), conv_fp(matrix[7]), conv_fp(matrix[8]));

                // Calculate scale factors
                let scale_0 = (conv_fp(matrix[0]).powi(2) + conv_fp(matrix[3]).powi(2)).sqrt();
                let scale_1 = (conv_fp(matrix[1]).powi(2) + conv_fp(matrix[4]).powi(2)).sqrt();

                println!("[DEBUG] Scale factors: scale_0 = {}, scale_1 = {}", scale_0, scale_1);

                // Normalize matrix elements
                let a = if scale_0 > 0.0 { conv_fp(matrix[0]) / scale_0 } else { conv_fp(matrix[0]) };
                let b = if scale_1 > 0.0 { conv_fp(matrix[1]) / scale_1 } else { conv_fp(matrix[1]) };

                // Calculate rotation angle: atan2(b, a)
                // Note: FFmpeg uses counter-clockwise as positive, so we negate
                let rotation = -b.atan2(a) * 180.0 / std::f64::consts::PI;

                println!("[DEBUG] Normalized: a = {}, b = {}", a, b);
                println!("[DEBUG] Raw rotation (radians): {}", b.atan2(a));
                println!("[DEBUG] Calculated rotation (degrees): {}", rotation);

                return Some(rotation.round() as i32);
            }
        }
    }

    println!("[DEBUG] No rotation metadata found");
    None
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_video> [output.jpg] [rotation]", args[0]);
        eprintln!("  input_video: Path to video file (mp4, mov, avi, mkv, etc.)");
        eprintln!("  output.jpg: Output JPEG path (default: same name with _thumb.jpg)");
        eprintln!("  rotation: Optional rotation in degrees (90, -90, 180, etc.)");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from(input_path).with_extension("jpg")
    };

    // Optional manual rotation override for testing
    let manual_rotation = args.get(3).and_then(|r| r.parse::<i32>().ok());

    println!("Input: {}", input_path);
    println!("Output: {}", output_path.display());
    if let Some(rot) = manual_rotation {
        println!("Manual rotation override: {} degrees", rot);
    }

    // Initialize FFmpeg
    println!("Initializing FFmpeg...");
    ffmpeg_next::init().expect("Failed to initialize FFmpeg");

    // Open video file
    println!("Opening video file...");
    let mut ictx = input(input_path).expect("Failed to open video file");

    // Find video stream
    let video_stream = ictx
        .streams()
        .best(Type::Video)
        .expect("No video stream found");
    let video_index = video_stream.index();

    println!("Video stream index: {}", video_index);

    // Get video properties
    let decoder_ctx = Context::from_parameters(video_stream.parameters())
        .expect("Failed to create decoder context");
    let mut decoder = decoder_ctx.decoder().video()
        .expect("Failed to create video decoder");

    println!("Video dimensions: {}x{}", decoder.width(), decoder.height());
    println!("Video format: {:?}", decoder.format());

    // Get rotation angle from video stream, or use manual override
    let rotation = manual_rotation.or_else(|| get_rotation_angle(&video_stream));
    println!("Rotation angle: {:?}", rotation);

    // Determine if aspect ratio needs to be swapped for target size calculation
    // 90, -90, 270, -270 degree rotations swap width and height visually
    let needs_swap = matches!(rotation, Some(r) if r == 90 || r == -90 || r == 270 || r == -270);

    // Use original decoder dimensions for scaler - the decoder outputs in original dimensions
    // Only swap aspect ratio for target size calculation when rotation is 90/-90
    let (scaler_width, scaler_height) = (decoder.width(), decoder.height());
    let (target_width, target_height) = if needs_swap {
        // For 90/-90 rotation, the visual aspect ratio is swapped
        let aspect_ratio = scaler_width as f64 / scaler_height as f64;
        let target_h = (600.0 / aspect_ratio) as u32;
        println!("Using swapped aspect ratio: 600x{}", target_h);
        (600, target_h)
    } else {
        let aspect_ratio = scaler_height as f64 / scaler_width as f64;
        let target_h = (600.0 * aspect_ratio) as u32;
        println!("Using normal aspect ratio: 600x{}", target_h);
        (600, target_h)
    };

    // Seek to target time (default 1.0 second)
    let offset_seconds = 1.0;
    println!("Seeking to {} seconds...", offset_seconds);

    // Seek to position - FFmpeg will find the closest keyframe
    let timestamp = (offset_seconds * 1_000_000.0) as i64;

    if let Err(e) = ictx.seek(timestamp, ..timestamp) {
        println!("Seek warning (may still work): {:?}", e);
    }

    // Create scaler for converting to RGB24
    // Always use original decoder dimensions for input
    println!("Scaling to {}x{}", target_width, target_height);

    let mut scaler = ScalingContext::get(
        decoder.format(),
        scaler_width,
        scaler_height,
        Pixel::RGB24,
        target_width,
        target_height,
        Flags::BILINEAR,
    ).expect("Failed to create scaler");

    // Decode frames until we get one
    let mut frame_found = false;
    let mut rgb_frame = Video::empty();

    for (stream_idx, packet) in ictx.packets() {
        if stream_idx.index() == video_index {
            if let Err(e) = decoder.send_packet(&packet) {
                println!("Send packet error: {:?}", e);
                continue;
            }

            let mut decoded = Video::empty();
            while let Ok(_) = decoder.receive_frame(&mut decoded) {
                if let Err(e) = scaler.run(&decoded, &mut rgb_frame) {
                    println!("Scaler error: {:?}", e);
                    continue;
                }

                println!("Frame decoded at PTS: {:?}", decoded.pts());
                frame_found = true;
                break;
            }

            if frame_found {
                break;
            }
        }
    }

    if !frame_found {
        let _ = decoder.send_eof();
        let mut decoded = Video::empty();
        while let Ok(_) = decoder.receive_frame(&mut decoded) {
            if let Err(e) = scaler.run(&decoded, &mut rgb_frame) {
                println!("Scaler error (flush): {:?}", e);
                continue;
            }
            println!("Frame decoded after EOF at PTS: {:?}", decoded.pts());
            frame_found = true;
            break;
        }
    }

    if !frame_found {
        eprintln!("Failed to decode any frame from video");
        std::process::exit(1);
    }

    // Get RGB data and handle stride padding
    let width = rgb_frame.width() as u32;
    let height = rgb_frame.height() as u32;
    let data = rgb_frame.data(0);
    let stride = rgb_frame.stride(0);
    let bytes_per_row = (width * 3) as usize;

    println!("RGB frame: {}x{}, data size: {}, stride: {}", width, height, data.len(), stride);

    // Create RGB image, handling stride padding if necessary
    let rgb_image = if stride == 0 || stride == bytes_per_row {
        // Data is tightly packed (or stride not available), use directly
        image::RgbImage::from_raw(width, height, data.to_vec())
            .expect("Failed to create image from RGB data")
    } else if stride > bytes_per_row {
        // Data has padding, need to copy row by row to remove padding
        let rgb_data: Vec<u8> = (0..height as usize)
            .flat_map(|row| {
                let row_offset = row * stride;
                data[row_offset..row_offset + bytes_per_row].to_vec()
            })
            .collect();

        image::RgbImage::from_raw(width, height, rgb_data)
            .expect("Failed to create image from RGB data")
    } else {
        // Stride is less than expected (shouldn't happen), try to use as-is
        image::RgbImage::from_raw(width, height, data.to_vec())
            .expect("Failed to create image from RGB data")
    };

    // Apply rotation if needed
    let normalized_rotation = rotation.map(|r| r.rem_euclid(360));
    println!("Normalized rotation: {:?}", normalized_rotation);

    let final_image = match normalized_rotation {
        Some(90) | Some(270) => {
            println!("Applying 90-degree rotation");
            image::imageops::rotate90(&rgb_image)
        }
        Some(180) => {
            println!("Applying 180-degree rotation");
            image::imageops::rotate180(&rgb_image)
        }
        Some(0) | None => {
            println!("No rotation needed");
            rgb_image
        }
        _ => {
            println!("Unsupported rotation angle: {:?}", rotation);
            rgb_image
        }
    };

    // Encode as JPEG
    println!("Encoding to JPEG...");
    let mut jpeg_bytes = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
        &mut jpeg_bytes,
        80, // Quality 80%
    );
    encoder.encode_image(&final_image)
        .expect("Failed to encode JPEG");

    // Save to file
    println!("Saving to {}...", output_path.display());
    std::fs::write(&output_path, &jpeg_bytes)
        .expect("Failed to write output file");

    println!("Success! Output size: {} bytes", jpeg_bytes.len());
}
