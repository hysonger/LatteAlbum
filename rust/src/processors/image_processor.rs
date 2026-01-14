use crate::processors::processor_trait::{MediaMetadata, MediaProcessor, MediaType, ProcessingError};
use async_trait::async_trait;
use std::path::Path;

/// Standard image processor for JPEG, PNG, GIF, WebP, TIFF, BMP
pub struct StandardImageProcessor;

impl StandardImageProcessor {
    pub fn new() -> Self {
        Self
    }

    const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff"];
}

#[async_trait]
impl MediaProcessor for StandardImageProcessor {
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
        MediaType::Image
    }

    async fn process(&self, path: &Path) -> Result<MediaMetadata, ProcessingError> {
        let mut metadata = MediaMetadata::default();

        // Get file size
        if let Ok(metadata_file) = path.metadata() {
            metadata.file_size = Some(metadata_file.len() as i64);
        }

        // Get dimensions
        let (width, height) = get_image_dimensions(path)?;
        metadata.width = Some(width as i32);
        metadata.height = Some(height as i32);

        // Set MIME type
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            metadata.mime_type = Some(match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" => "image/jpeg".to_string(),
                "png" => "image/png".to_string(),
                "gif" => "image/gif".to_string(),
                "webp" => "image/webp".to_string(),
                "tiff" => "image/tiff".to_string(),
                "bmp" => "image/bmp".to_string(),
                _ => "image/jpeg".to_string(),
            });
        }

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
            use image::ImageReader;

            let img = ImageReader::open(path)?.decode()?;

            let ratio = img.height() as f64 / img.width() as f64;
            let target_height = (target_width as f64 * ratio) as u32;

            let thumbnail = img
                .resize(target_width, target_height, image::imageops::FilterType::Lanczos3)
                .to_rgb8();

            let mut bytes = Vec::new();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut bytes,
                (quality * 100.0) as u8,
            );
            encoder.encode_image(&thumbnail)?;

            Ok(Some(bytes))
        })
        .await
        .map_err(|e| ProcessingError::Processing(e.to_string()))?
    }
}

fn get_image_dimensions(path: &Path) -> Result<(u32, u32), ProcessingError> {
    use image::{ImageReader, GenericImageView};

    let img = ImageReader::open(path)?.decode()?;
    Ok(img.dimensions())
}
