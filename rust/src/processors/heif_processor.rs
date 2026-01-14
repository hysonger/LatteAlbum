use crate::processors::processor_trait::{
    MediaMetadata, MediaProcessor, MediaType, ProcessingError,
};
use async_trait::async_trait;
use std::path::Path;

/// HEIF/HEIC image processor
/// Uses the image crate's built-in HEIF support
pub struct HeifImageProcessor;

impl HeifImageProcessor {
    pub fn new() -> Self {
        Self
    }

    const SUPPORTED_EXTENSIONS: &[&str] = &["heic", "heif"];
}

#[async_trait]
impl MediaProcessor for HeifImageProcessor {
    fn supports(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Self::SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str())
        } else {
            false
        }
    }

    fn priority(&self) -> i32 {
        100 // Higher priority than standard image processor
    }

    fn media_type(&self) -> MediaType {
        MediaType::Heif
    }

    async fn process(&self, path: &Path) -> Result<MediaMetadata, ProcessingError> {
        let mut metadata = MediaMetadata::default();

        // Get file size
        if let Ok(metadata_file) = path.metadata() {
            metadata.file_size = Some(metadata_file.len() as i64);
        }

        // Try to decode HEIF image to get dimensions
        // The image crate has experimental HEIF support
        let path = path.to_path_buf();
        let dimensions = tokio::task::spawn_blocking(move || {
            use image::GenericImageView;

            let img = image::io::Reader::open(path)
                .map_err(|e| ProcessingError::Processing(e.to_string()))?
                .with_guessed_format()
                .map_err(|e| ProcessingError::Processing(e.to_string()))?
                .decode()
                .map_err(|e| ProcessingError::Processing(e.to_string()))?;
            Ok::<(u32, u32), ProcessingError>(img.dimensions())
        })
        .await
        .map_err(|e| ProcessingError::Processing(e.to_string()))??;

        metadata.width = Some(dimensions.0 as i32);
        metadata.height = Some(dimensions.1 as i32);
        metadata.mime_type = Some("image/heic".to_string());

        Ok(metadata)
    }

    async fn generate_thumbnail(
        &self,
        path: &Path,
        target_width: u32,
        quality: f32,
    ) -> Result<Option<Vec<u8>>, ProcessingError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let img = image::io::Reader::open(path)
                .map_err(|e| ProcessingError::Processing(e.to_string()))?
                .with_guessed_format()
                .map_err(|e| ProcessingError::Processing(e.to_string()))?
                .decode()
                .map_err(|e| ProcessingError::Processing(e.to_string()))?;

            let ratio = img.height() as f64 / img.width() as f64;
            let target_height = (target_width as f64 * ratio) as u32;

            let thumbnail = img
                .resize(target_width, target_height, image::imageops::FilterType::Lanczos3)
                .to_rgb8();

            let mut bytes = Vec::new();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, (quality * 100.0) as u8);
            encoder
                .encode_image(&thumbnail)
                .map_err(|e| ProcessingError::Processing(e.to_string()))?;

            Ok(Some(bytes))
        })
        .await
        .map_err(|e| ProcessingError::Processing(e.to_string()))?
    }
}
