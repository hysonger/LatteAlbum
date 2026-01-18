use crate::processors::processor_trait::{
    MediaMetadata, MediaProcessor, MediaType, ProcessingError,
};
use async_trait::async_trait;
use std::path::Path;

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
        _target_width: u32,
        _quality: f32,
    ) -> Result<Option<Vec<u8>>, ProcessingError> {
        #[cfg(feature = "video-processing")]
        {
            let path = path.to_path_buf();
            let ffmpeg_path = self.ffmpeg_path.clone();

            return tokio::task::spawn_blocking(move || {
                generate_video_thumbnail(&path, target_width, ffmpeg_path.as_deref())
            })
            .await
            .map_err(|e| ProcessingError::Processing(e.to_string()))?;
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

    let input = input(path).map_err(|e| ProcessingError::ExternalTool(e.to_string()))?;

    let mut width = None;
    let mut height = None;
    let mut duration = None;
    let mut codec = None;

    // Get stream information
    for stream in input.streams() {
        if stream.codec().medium() == ffmpeg_next::media::Type::Video {
            let codec_ctx = stream.codec();
            width = Some(codec_ctx.width() as i32);
            height = Some(codec_ctx.height() as i32);
            codec = codec_ctx.name().map(|s| s.to_string());

            // Get duration from stream
            if let Ok(d) = stream.duration() {
                let time_base = stream.time_base();
                duration = Some(d as f64 * time_base.numer() as f64 / time_base.denom() as f64);
            }
        }
    }

    // Get duration from format if not found in stream
    if duration.is_none() {
        if let Ok(d) = input.duration() {
            duration = Some(d as f64 / 1_000_000.0); // Convert from microseconds
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
    // Simplified video thumbnail generation
    // In a full implementation, this would use FFmpeg to extract a frame
    // For now, return an empty placeholder
    Ok(vec![])
}
