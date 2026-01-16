use crate::processors::image_processor::extract_exif;
use crate::processors::processor_trait::{
    MediaMetadata, MediaProcessor, MediaType, ProcessingError,
};
use async_trait::async_trait;
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use std::path::Path;

/// HEIF/HEIC image processor
/// Uses libheif-rs for HEIC decoding
pub struct HeifImageProcessor;

impl Default for HeifImageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

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

        // Use libheif-rs to read HEIC dimensions (format-specific)
        let path_buf = path.to_path_buf();
        let dimensions = tokio::task::spawn_blocking(move || {
            let path_str = path_buf.to_string_lossy();
            let ctx = HeifContext::read_from_file(&path_str)
                .map_err(|e| ProcessingError::Processing(e.to_string()))?;
            let handle = ctx.primary_image_handle()
                .map_err(|e| ProcessingError::Processing(e.to_string()))?;
            Ok::<(u32, u32), ProcessingError>((handle.width(), handle.height()))
        })
        .await
        .map_err(|e| ProcessingError::Processing(e.to_string()))??;

        metadata.width = Some(dimensions.0 as i32);
        metadata.height = Some(dimensions.1 as i32);
        metadata.mime_type = Some("image/heic".to_string());

        // Extract EXIF metadata (supports HEIC via kamadak-exif)
        extract_exif(path, &mut metadata);

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
            // Read HEIC file using libheif-rs
            let path_str = path.to_string_lossy();
            let ctx = HeifContext::read_from_file(&path_str)
                .map_err(|e| ProcessingError::Processing(e.to_string()))?;
            let handle = ctx.primary_image_handle()
                .map_err(|e| ProcessingError::Processing(e.to_string()))?;

            // Decode to RGBA
            // HEIC 文件使用 YCbCr 颜色空间，libheif 解码时使用 Rgba 会自动转换
            let lib_heif = LibHeif::new();
            let image = lib_heif.decode(
                &handle,
                ColorSpace::Rgb(RgbChroma::Rgba),
                None,
            ).map_err(|e| ProcessingError::Processing(e.to_string()))?;

            // If target_width is 0, use full size (no resize)
            // Otherwise scale to target dimensions
            let scaled = if target_width == 0 {
                // Full size - use original dimensions
                image
            } else {
                // Calculate target height maintaining aspect ratio
                let ratio = image.height() as f64 / image.width() as f64;
                let target_height = (target_width as f64 * ratio) as u32;

                // Scale if needed
                if image.width() > target_width || image.height() > target_height {
                    image.scale(target_width, target_height, None)
                        .map_err(|e| ProcessingError::Processing(e.to_string()))?
                } else {
                    image
                }
            };

            // Get interleaved RGBA data
            let planes = scaled.planes();
            let interleaved = planes.interleaved
                .as_ref()
                .ok_or_else(|| ProcessingError::Processing("No interleaved plane in HEIC".to_string()))?;

            // Handle stride vs width difference (for memory alignment padding)
            let width = interleaved.width;
            let height = interleaved.height;
            let stride = interleaved.stride;
            let data = &interleaved.data;

            // Create RgbaImage from raw data, handling stride padding if necessary
            // interleaved 数据是 4 通道 (R, G, B, A)，不是 3 通道
            let rgba_image = if stride == width as usize * 4 {
                // Data is tightly packed, can use directly
                image::RgbaImage::from_raw(width, height, data.to_vec())
                    .ok_or_else(|| ProcessingError::Processing("Failed to create image from HEIC data".to_string()))?
            } else {
                // Data has padding, need to copy row by row
                let mut rgb_data = Vec::with_capacity(((width as usize * height as usize * 4)));
                let bytes_per_row = width as usize * 4;
                for row in 0..height as usize {
                    let row_offset = row * stride;
                    rgb_data.extend_from_slice(&data[row_offset..row_offset + bytes_per_row]);
                }
                image::RgbaImage::from_raw(width, height, rgb_data)
                    .ok_or_else(|| ProcessingError::Processing("Failed to create image from HEIC data".to_string()))?
            };

            // RGBA to RGB conversion (discard alpha channel)
            // JPEG encoder requires 3-channel RGB data
            let rgb_image = image::DynamicImage::ImageRgba8(rgba_image).to_rgb8();

            // Encode as JPEG
            let mut jpeg_bytes = Vec::new();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut jpeg_bytes,
                (quality * 100.0) as u8,
            );
            encoder.encode_image(&rgb_image)
                .map_err(|e| ProcessingError::Processing(e.to_string()))?;

            Ok(Some(jpeg_bytes))
        })
        .await
        .map_err(|e| ProcessingError::Processing(e.to_string()))?
    }
}
