use crate::processors::processor_trait::{
    MediaMetadata, MediaProcessor, MediaType, ProcessingError,
};
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "video-processing")]
use ffmpeg_next::codec::packet::side_data::Type as PacketSideDataType;

/// Get rotation angle from video stream's side_data (DisplayMatrix)
#[cfg(feature = "video-processing")]
fn get_rotation_angle(stream: &ffmpeg_next::Stream) -> Option<i32> {
    for side_data in stream.side_data() {
        if side_data.kind() == PacketSideDataType::DisplayMatrix {
            let data = side_data.data();

            // DisplayMatrix is a 3x3 matrix of int32_t (9 x 4 = 36 bytes)
            // Format: 16.16 fixed-point representation
            // Layout: [a, b, u, c, d, v, x, y, w] representing a 3x3 matrix
            if data.len() >= 36 {
                let matrix: &[i32] = unsafe {
                    std::slice::from_raw_parts(
                        data.as_ptr() as *const i32,
                        9
                    )
                };

                // Convert from fixed-point (16.16) to floating-point
                let conv_fp = |x: i32| x as f64 / (1i32 << 16) as f64;

                // Calculate scale factors
                let scale_0 = (conv_fp(matrix[0]).powi(2) + conv_fp(matrix[3]).powi(2)).sqrt();
                let scale_1 = (conv_fp(matrix[1]).powi(2) + conv_fp(matrix[4]).powi(2)).sqrt();

                // Normalize matrix elements
                let a = if scale_0 > 0.0 { conv_fp(matrix[0]) / scale_0 } else { conv_fp(matrix[0]) };
                let b = if scale_1 > 0.0 { conv_fp(matrix[1]) / scale_1 } else { conv_fp(matrix[1]) };

                // Calculate rotation angle: atan2(b, a)
                // Note: FFmpeg uses counter-clockwise as positive, so we negate
                let rotation = -b.atan2(a) * 180.0 / std::f64::consts::PI;

                tracing::debug!("DisplayMatrix rotation: {} degrees", rotation);
                return Some(rotation.round() as i32);
            }
        }
    }

    None
}

/// Video processor for MP4, AVI, MOV, MKV, etc.
/// Uses ffmpeg-next for video processing when available
pub struct VideoProcessor {
    #[allow(dead_code)]
    ffmpeg_path: Option<String>,
}

impl VideoProcessor {
    pub fn new(ffmpeg_path: Option<String>) -> Self {
        Self { ffmpeg_path }
    }

    const SUPPORTED_EXTENSIONS: &[&str] = &["mp4", "avi", "mov", "mkv", "wmv", "flv", "webm"];
}

#[async_trait]
impl MediaProcessor for VideoProcessor {
    fn supports(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Self::SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str())
        } else {
            false
        }
    }

    fn priority(&self) -> i32 {
        10
    }

    fn media_type(&self) -> MediaType {
        MediaType::Video
    }

    async fn process(&self, path: &Path) -> Result<MediaMetadata, ProcessingError> {
        let mut metadata = MediaMetadata::default();

        #[cfg(feature = "video-processing")]
        {
            // Try to extract video metadata using FFmpeg (format-specific)
            match extract_video_metadata(path) {
                Ok((width, height, duration, codec)) => {
                    metadata.width = width;
                    metadata.height = height;
                    metadata.duration = duration;
                    metadata.video_codec = codec;
                }
                Err(e) => {
                    tracing::warn!("Failed to extract video metadata: {}", e);
                }
            }
        }

        #[cfg(not(feature = "video-processing"))]
        {
            tracing::warn!("Video processing not enabled - skipping metadata extraction for {}", path.display());
        }

        // Set MIME type
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            metadata.mime_type = Some(match ext.to_lowercase().as_str() {
                "mp4" => "video/mp4".to_string(),
                "mov" => "video/quicktime".to_string(),
                "avi" => "video/x-msvideo".to_string(),
                "mkv" => "video/x-matroska".to_string(),
                "webm" => "video/webm".to_string(),
                "wmv" => "video/x-ms-wmv".to_string(),
                "flv" => "video/x-flv".to_string(),
                _ => "video/mp4".to_string(),
            });
        }

        Ok(metadata)
    }

    async fn generate_thumbnail(
        &self,
        path: &Path,
        _target_size: u32,
        _quality: f32,
        _fit_to_height: bool,
    ) -> Result<Option<Vec<u8>>, ProcessingError> {
        #[cfg(feature = "video-processing")]
        {
            let path = path.to_path_buf();
            let ffmpeg_path = self.ffmpeg_path.clone();

            let result = tokio::task::spawn_blocking(move || {
                generate_video_thumbnail(&path, _target_size, ffmpeg_path.as_deref())
            })
            .await
            .map_err(|e| ProcessingError::Processing(e.to_string()))?;

            return result.map(Some).map_err(|e| ProcessingError::Processing(e.to_string()));
        }

        #[cfg(not(feature = "video-processing"))]
        {
            tracing::warn!("Video processing not enabled - cannot generate thumbnail for {}", path.display());
        }

        Ok(None)
    }
}

#[cfg(feature = "video-processing")]
fn extract_video_metadata(path: &Path) -> Result<(Option<i32>, Option<i32>, Option<f64>, Option<String>), ProcessingError> {
    use ffmpeg_next::format::input;
    use ffmpeg_next::codec::context::Context;
    use ffmpeg_next::media::Type;

    let input = input(path).map_err(|e| ProcessingError::ExternalTool(e.to_string()))?;

    let mut width = None;
    let mut height = None;
    let mut duration = None;
    let mut codec = None;

    // Get stream information
    for stream in input.streams() {
        // Check if this is a video stream by checking frames
        if stream.frames() > 0 {
            // Get dimensions from decoder
            if let Ok(params) = Context::from_parameters(stream.parameters()) {
                if let Ok(decoder) = params.decoder().video() {
                    width = Some(decoder.width() as i32);
                    height = Some(decoder.height() as i32);
                    // Get codec name
                    let codec_id = decoder.id();
                    codec = Some(codec_id.name().to_string());
                }
            }

            // Get duration from stream (returns i64 directly in new API)
            let dur = stream.duration();
            if dur > 0 {
                let time_base = stream.time_base();
                duration = Some(dur as f64 * time_base.numerator() as f64 / time_base.denominator() as f64);
            }
        }
    }

    // Get duration from format if not found in stream
    if duration.is_none() {
        let dur = input.duration();
        if dur > 0 {
            duration = Some(dur as f64 / 1_000_000.0); // Convert from microseconds
        }
    }

    Ok((width, height, duration, codec))
}

#[cfg(feature = "video-processing")]
fn generate_video_thumbnail(
    path: &Path,
    target_width: u32,
    _ffmpeg_path: Option<&str>,
) -> Result<Vec<u8>, ProcessingError> {
    use ffmpeg_next::format::input;
    use ffmpeg_next::media::Type;
    use ffmpeg_next::codec::context::Context;
    use ffmpeg_next::software::scaling::{Context as ScalingContext, Flags};
    use ffmpeg_next::format::Pixel;
    use ffmpeg_next::util::frame::video::Video;

    // Initialize FFmpeg
    if let Err(e) = ffmpeg_next::init() {
        tracing::warn!("Failed to initialize FFmpeg: {}", e);
        return Err(ProcessingError::ExternalTool(e.to_string()));
    }

    // Open video file
    let mut ictx = match input(path) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::warn!("Failed to open video file: {}", e);
            return Err(ProcessingError::ExternalTool(e.to_string()));
        }
    };

    // Find video stream
    let video_stream = match ictx.streams().best(Type::Video) {
        Some(stream) => stream,
        None => {
            tracing::warn!("No video stream found");
            return Err(ProcessingError::Processing("No video stream found".to_string()));
        }
    };

    // Get video index early to avoid borrow conflicts
    let video_index = video_stream.index();

    // Create decoder first to get original dimensions
    let decoder_ctx = match Context::from_parameters(video_stream.parameters()) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::warn!("Failed to create decoder context: {}", e);
            return Err(ProcessingError::ExternalTool(e.to_string()));
        }
    };

    let mut decoder = match decoder_ctx.decoder().video() {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("Failed to create video decoder: {}", e);
            return Err(ProcessingError::ExternalTool(e.to_string()));
        }
    };

    // Get rotation angle from video stream
    let rotation = get_rotation_angle(&video_stream);

    // Determine if aspect ratio needs to be swapped for target size calculation
    // 90, -90, 270, -270 degree rotations swap width and height visually
    let needs_swap = matches!(rotation, Some(r) if r == 90 || r == -90 || r == 270 || r == -270);

    // Use original decoder dimensions for scaler
    let (scaler_width, scaler_height) = (decoder.width(), decoder.height());
    let (target_width, target_height) = if needs_swap {
        // For 90/-90 rotation, the visual aspect ratio is swapped
        let aspect_ratio = scaler_width as f64 / scaler_height as f64;
        let target_h = (target_width as f64 / aspect_ratio) as u32;
        (target_width, target_h)
    } else {
        let aspect_ratio = scaler_height as f64 / scaler_width as f64;
        let target_h = (target_width as f64 * aspect_ratio) as u32;
        (target_width, target_h)
    };

    // Seek to target time (default 1.0 second)
    let offset_seconds = 1.0;
    let timestamp = (offset_seconds * 1_000_000.0) as i64;

    // Try to seek, ignore errors as we can still decode from start
    let _ = ictx.seek(timestamp, ..timestamp);

    // Create scaler for converting to RGB24 - always use original decoder dimensions
    let mut scaler = match ScalingContext::get(
        decoder.format(),
        scaler_width,
        scaler_height,
        Pixel::RGB24,
        target_width,
        target_height,
        Flags::BILINEAR,
    ) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Failed to create scaler: {}", e);
            return Err(ProcessingError::ExternalTool(e.to_string()));
        }
    };
    let mut frame_found = false;
    let mut rgb_frame = Video::empty();

    // Decode packets until we get a frame
    for (stream_idx, packet) in ictx.packets() {
        if stream_idx.index() == video_index {
            if let Err(e) = decoder.send_packet(&packet) {
                continue;
            }

            let mut decoded = Video::empty();
            while let Ok(_) = decoder.receive_frame(&mut decoded) {
                if scaler.run(&decoded, &mut rgb_frame).is_ok() {
                    frame_found = true;
                    break;
                }
            }

            if frame_found {
                break;
            }
        }
    }

    // Try EOF flush if no frame found
    if !frame_found {
        let _ = decoder.send_eof();
        let mut decoded = Video::empty();
        while let Ok(_) = decoder.receive_frame(&mut decoded) {
            if scaler.run(&decoded, &mut rgb_frame).is_ok() {
                frame_found = true;
                break;
            }
        }
    }

    if !frame_found {
        tracing::warn!("Failed to decode any frame from video");
        return Err(ProcessingError::Processing("Failed to decode video frame".to_string()));
    }

    // Get RGB data and handle stride padding
    let width = rgb_frame.width() as u32;
    let height = rgb_frame.height() as u32;
    let data = rgb_frame.data(0);
    let stride = rgb_frame.stride(0);
    let bytes_per_row = (width * 3) as usize;

    // Create RGB image, handling stride padding if necessary
    let rgb_image = if stride == 0 || stride == bytes_per_row {
        // Data is tightly packed (or stride not available), use directly
        image::RgbImage::from_raw(width, height, data.to_vec())
            .ok_or_else(|| ProcessingError::Processing("Failed to create image from RGB data".to_string()))?
    } else if stride > bytes_per_row {
        // Data has padding, need to copy row by row to remove padding
        let rgb_data: Vec<u8> = (0..height as usize)
            .flat_map(|row| {
                let row_offset = row * stride;
                data[row_offset..row_offset + bytes_per_row].to_vec()
            })
            .collect();

        image::RgbImage::from_raw(width, height, rgb_data)
            .ok_or_else(|| ProcessingError::Processing("Failed to create image from RGB data".to_string()))?
    } else {
        // Stride is less than expected (shouldn't happen), try to use as-is
        image::RgbImage::from_raw(width, height, data.to_vec())
            .ok_or_else(|| ProcessingError::Processing("Failed to create image from RGB data".to_string()))?
    };

    // Apply rotation if needed
    let normalized_rotation = rotation.map(|r| r.rem_euclid(360));

    let final_image = match normalized_rotation {
        Some(90) => {
            // DisplayMatrix 90° = counter-clockwise 90° = rotate270
            image::imageops::rotate270(&rgb_image)
        }
        Some(270) => {
            // DisplayMatrix 270° (-90°) = clockwise 90° = rotate90
            image::imageops::rotate90(&rgb_image)
        }
        Some(180) => {
            image::imageops::rotate180(&rgb_image)
        }
        Some(0) | None => {
            rgb_image
        }
        _ => {
            // Unsupported rotation angle, return as-is
            rgb_image
        }
    };

    // Encode as JPEG with 80% quality
    let mut jpeg_bytes = Vec::new();
    {
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_bytes, 80);
        if let Err(e) = encoder.encode_image(&final_image) {
            tracing::warn!("Failed to encode JPEG: {}", e);
            return Err(ProcessingError::Processing(e.to_string()));
        }
    }

    Ok(jpeg_bytes)
}
