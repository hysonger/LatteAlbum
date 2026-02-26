//! Video thumbnail generation test utility using ffmpeg-next library
//!
//! Usage: cargo run --features vendor-build --example video_thumbnail_with_rotation <input_video> [output.jpg]
//!
//! Extracts a frame from video at specified offset and converts to JPEG.

use ffmpeg_next::format::input;
use ffmpeg_next::media::Type;
use ffmpeg_next::codec::context::Context;
use ffmpeg_next::software::scaling::{Context as ScalingContext, Flags};
use ffmpeg_next::format::Pixel;
use ffmpeg_next::util::frame::video::Video;
use ffmpeg_next::util::frame::side_data::Type as SideDataType;
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_video> [output.jpg]", args[0]);
        eprintln!("  input_video: Path to video file (mp4, mov, avi, mkv, etc.)");
        eprintln!("  output.jpg: Output JPEG path (default: same name with _thumb.jpg)");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from(input_path).with_extension("jpg")
    };

    println!("Input: {}", input_path);
    println!("Output: {}", output_path.display());

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

    // Seek to target time (default 1.0 second)
    let offset_seconds = 1.0;
    println!("Seeking to {} seconds...", offset_seconds);

    // Seek to position - FFmpeg will find the closest keyframe
    let timestamp = (offset_seconds * 1_000_000.0) as i64;

    if let Err(e) = ictx.seek(timestamp, ..timestamp) {
        println!("Seek warning (may still work): {:?}", e);
    }

    // Create scaler for converting to RGB24
    let target_width = 600;
    let aspect_ratio = decoder.height() as f64 / decoder.width() as f64;
    let target_height = (target_width as f64 * aspect_ratio) as u32;

    println!("Scaling to {}x{}", target_width, target_height);

    let mut scaler = ScalingContext::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
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

    // Get RGB data
    let width = rgb_frame.width() as u32;
    let height = rgb_frame.height() as u32;
    let data = rgb_frame.data(0);

    println!("RGB frame: {}x{}, data size: {}", width, height, data.len());

    // Create image::RgbImage from the raw RGB data
    let rgb_image = image::RgbImage::from_raw(width, height, data.to_vec())
        .expect("Failed to create image from RGB data");

    // Encode as JPEG
    println!("Encoding to JPEG...");
    let mut jpeg_bytes = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
        &mut jpeg_bytes,
        80, // Quality 80%
    );
    encoder.encode_image(&rgb_image)
        .expect("Failed to encode JPEG");

    // Save to file
    println!("Saving to {}...", output_path.display());
    std::fs::write(&output_path, &jpeg_bytes)
        .expect("Failed to write output file");

    println!("Success! Output size: {} bytes", jpeg_bytes.len());
}
