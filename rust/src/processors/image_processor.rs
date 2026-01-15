use crate::processors::processor_trait::{MediaMetadata, MediaProcessor, MediaType, ProcessingError};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use std::path::Path;
use std::time::UNIX_EPOCH;

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

            // Extract create time
            if let Ok(created) = metadata_file.created() {
                if let Ok(duration) = duration_since_unix_epoch(created) {
                    if let Some(ts) = NaiveDateTime::from_timestamp_opt(duration.as_secs() as i64, 0) {
                        metadata.create_time = Some(ts);
                    }
                }
            }

            // Extract modify time
            if let Ok(modified) = metadata_file.modified() {
                if let Ok(duration) = duration_since_unix_epoch(modified) {
                    if let Some(ts) = NaiveDateTime::from_timestamp_opt(duration.as_secs() as i64, 0) {
                        metadata.modify_time = Some(ts);
                    }
                }
            }
        }

        // Get dimensions
        let (width, height) = get_image_dimensions(path)?;
        metadata.width = Some(width as i32);
        metadata.height = Some(height as i32);

        // Extract EXIF metadata for all supported image formats
        extract_jpeg_exif(path, &mut metadata);

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

/// Convert SystemTime to Duration since Unix epoch
fn duration_since_unix_epoch(time: std::time::SystemTime) -> Result<std::time::Duration, std::time::SystemTimeError> {
    time.duration_since(UNIX_EPOCH)
}

/// Extract EXIF metadata from JPEG files
fn extract_jpeg_exif(path: &Path, metadata: &mut MediaMetadata) {
    use exif::Reader;

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };

    // Use Reader to parse EXIF data from the image file
    let exif = match Reader::new().read_from_container(&mut std::io::BufReader::new(file)) {
        Ok(e) => e,
        Err(_) => return,
    };

    for field in exif.fields() {
        let tag = field.tag;
        let value_str = field.value.display_as(tag).to_string().trim().to_string();

        match tag {
            exif::Tag::DateTimeOriginal | exif::Tag::DateTimeDigitized => {
                if let Ok(ts) = NaiveDateTime::parse_from_str(&value_str, "%Y:%m:%d %H:%M:%S") {
                    if metadata.exif_timestamp.is_none() || tag == exif::Tag::DateTimeOriginal {
                        metadata.exif_timestamp = Some(ts);
                    }
                }
            }
            exif::Tag::Model => {
                if !value_str.is_empty() {
                    metadata.camera_model = Some(value_str);
                }
            }
            exif::Tag::Make => {
                if !value_str.is_empty() {
                    metadata.camera_make = Some(value_str);
                }
            }
            _ => {}
        }
    }
}
