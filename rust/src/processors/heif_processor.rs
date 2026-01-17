use crate::processors::image_processor::extract_exif;
use crate::processors::processor_trait::{
    MediaMetadata, MediaProcessor, MediaType, ProcessingError,
};
use crate::services::TranscodingPool;
use async_trait::async_trait;
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use std::path::Path;
use std::sync::Arc;
use rayon::prelude::*;

/// HEIF/HEIC image processor
/// Uses libheif-rs for HEIC decoding
pub struct HeifImageProcessor {
    transcoding_pool: Option<Arc<TranscodingPool>>,
}

impl HeifImageProcessor {
    pub fn new(transcoding_pool: Option<Arc<TranscodingPool>>) -> Self {
        Self { transcoding_pool }
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
        let pool = self.transcoding_pool.clone();

        // Use transcoding pool if available, otherwise fallback to spawn_blocking
        if let Some(ref pool) = pool {
            // Run in transcoding pool (rayon thread)
            pool.scope(|_| {
                // Synchronous HEIC transcoding logic
                transcoding_generate_heic_thumbnail(&path, target_width, quality)
            })
        } else {
            // Fallback to spawn_blocking
            tokio::task::spawn_blocking(move || {
                transcoding_generate_heic_thumbnail(&path, target_width, quality)
            })
            .await
            .map_err(|e| ProcessingError::Processing(e.to_string()))?
        }
    }
}

/// Synchronous HEIC thumbnail generation for transcoding pool
fn transcoding_generate_heic_thumbnail(
    path: &Path,
    target_width: u32,
    quality: f32,
) -> Result<Option<Vec<u8>>, ProcessingError> {
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

    let width = interleaved.width;
    let height = interleaved.height;
    let stride = interleaved.stride;
    let bytes_per_row = width as usize * 4;

    // Create RgbaImage from raw data, handling stride padding if necessary
    // interleaved 数据是 4 通道 (R, G, B, A)，不是 3 通道
    let rgba_image = if stride == bytes_per_row {
        // Data is tightly packed, can use directly without stride copying
        image::RgbaImage::from_raw(width, height, interleaved.data.to_owned())
            .ok_or_else(|| ProcessingError::Processing("Failed to create image from HEIC data".to_string()))?
    } else {
        // Data has padding, need to copy row by row (remove padding)
        // Use regular iterator instead of par_iter for single-threaded rayon scope
        let rgb_data: Vec<u8> = (0..height as usize).into_iter()
            .flat_map(|row| {
                let row_offset = row * stride;
                interleaved.data[row_offset..row_offset + bytes_per_row].to_owned()
            }).collect();

        image::RgbaImage::from_raw(width, height, rgb_data)
            .ok_or_else(|| ProcessingError::Processing("Failed to create image from HEIC data".to_string()))?
    };

    // RGBA to RGB conversion (discard alpha channel)
    // JPEG encoder requires 3-channel RGB data
    let rgb_image = image::DynamicImage::ImageRgba8(rgba_image);

    // Encode as JPEG
    let mut jpeg_bytes = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
        &mut jpeg_bytes,
        (quality * 100.0) as u8,
    );
    encoder.encode_image(&rgb_image)
        .map_err(|e| ProcessingError::Processing(e.to_string()))?;

    Ok(Some(jpeg_bytes))
}
