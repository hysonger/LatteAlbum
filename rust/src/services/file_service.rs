use crate::db::{DatabasePool, MediaFileRepository};
use crate::processors::ProcessorRegistry;
use crate::services::{CacheService, TranscodingPool};
use bytes::Bytes;
use std::sync::Arc;
use tracing::debug;

/// Service for file operations
#[derive(Clone)]
pub struct FileService {
    db: DatabasePool,
    cache: Arc<CacheService>,
    processors: Arc<ProcessorRegistry>,
    #[allow(dead_code)]
    transcoding_pool: Arc<TranscodingPool>,
}

impl FileService {
    pub fn new(
        db: DatabasePool,
        cache: Arc<CacheService>,
        processors: Arc<ProcessorRegistry>,
        transcoding_pool: Arc<TranscodingPool>,
    ) -> Self {
        Self {
            db,
            cache,
            processors,
            transcoding_pool,
        }
    }
}

/// Service for file operations - methods
impl FileService {
    /// Get thumbnail for a file
    /// For "full" size (target_width == 0), browser-native formats are served directly without transcoding
    /// (JPEG, PNG, GIF, WebP, AVIF, SVG). Other formats like HEIC/HEIF will be transcoded.
    /// Returns (data, mime_type) tuple. For thumbnails, mime_type is "image/jpeg".
    pub async fn get_thumbnail(
        &self,
        file_id: &str,
        target_width: u32,
    ) -> Result<Option<(Vec<u8>, String)>, Box<dyn std::error::Error>> {
        // Check if this is a full-size request
        let is_full_size = target_width == 0;

        // Determine size label for caching
        let size_label = if is_full_size {
            "full".to_string()
        } else {
            match target_width {
                w if w <= 300 => "small".to_string(),
                w if w <= 450 => "medium".to_string(),
                _ => "large".to_string(),
            }
        };

        // For all sizes including full, check disk cache first
        if let Some(data) = self.cache.get_thumbnail(file_id, &size_label).await {
            // Thumbnails are always JPEG; full-size cache uses original format
            let mime_type = if is_full_size {
                guess_mime_type_from_path(file_id)
            } else {
                "image/jpeg".to_string()
            };
            // Convert Bytes to Vec<u8> for API compatibility
            return Ok(Some((data.to_vec(), mime_type)));
        }

        // Not in cache, generate thumbnail
        let repo = MediaFileRepository::new(&self.db);

        match repo.find_by_id(file_id).await {
            Ok(Some(file)) => {
                let path = std::path::Path::new(&file.file_path);
                if path.exists() {
                    // For full-size requests with browser-native formats, serve original file directly (no transcoding)
                    if is_full_size && is_browser_native_format(&file.file_name) {
                        if let Ok(data) = tokio::fs::read(path).await {
                            let mime_type = guess_mime_type(&file.file_name);
                            // Cache the data (Bytes::from takes ownership, so we clone for return)
                            let cache_data = Bytes::from(data.clone());
                            let _ = self.cache.put_thumbnail_bytes(file_id, &size_label, cache_data).await;
                            return Ok(Some((data, mime_type)));
                        }
                    }

                    // Generate thumbnail using processor (which uses transcoding_pool internally)
                    if let Some(processor) = self.processors.find_processor(path) {
                        match processor.generate_thumbnail(path, target_width, 0.8).await {
                            Ok(Some(thumbnail_data)) => {
                                // Cache the generated thumbnail (all sizes including full)
                                // Clone for caching since we need to return the original data
                                let cache_data = Bytes::from(thumbnail_data.clone());
                                let _ = self.cache.put_thumbnail_bytes(file_id, &size_label, cache_data).await;
                                return Ok(Some((thumbnail_data, "image/jpeg".to_string())));
                            }
                            Ok(None) => {
                                debug!("Processor returned no thumbnail for {}", file_id);
                            }
                            Err(e) => {
                                debug!("Failed to generate thumbnail for {}: {}", file_id, e);
                            }
                        }
                    }
                } else {
                    debug!("File not found: {}", file.file_path);
                }
            }
            Ok(None) => {
                debug!("File not found in database: {}", file_id);
            }
            Err(e) => {
                debug!("Database error when looking up file {}: {}", file_id, e);
            }
        }

        // Fallback: try to read original file as thumbnail for images
        if !is_full_size {
            self.generate_fallback_thumbnail(file_id).await
        } else {
            Ok(None)
        }
    }

    /// Generate a fallback thumbnail from the original file
    async fn generate_fallback_thumbnail(
        &self,
        file_id: &str,
    ) -> Result<Option<(Vec<u8>, String)>, Box<dyn std::error::Error>> {
        let repo = MediaFileRepository::new(&self.db);

        if let Ok(Some(file)) = repo.find_by_id(file_id).await {
            let path = std::path::Path::new(&file.file_path);
            if path.exists() {
                // For images, try to use the original file directly (scaled)
                if file.file_type == "image" {
                    let data = tokio::fs::read(path).await?;
                    // Basic JPEG/PNG check - if it's not a supported format, we can't serve it as thumbnail
                    let mime_type = if data.starts_with(&[0xFF, 0xD8]) {
                        "image/jpeg".to_string()
                    } else if data.starts_with(b"\x89PNG\r\n\x1a\n") {
                        "image/png".to_string()
                    } else {
                        return Ok(None);
                    };
                    return Ok(Some((data, mime_type)));
                }
            }
        }

        Ok(None)
    }

    /// Get original file content
    pub async fn get_original_file(
        &self,
        file_id: &str,
    ) -> Result<Option<(Vec<u8>, String)>, Box<dyn std::error::Error>> {
        let repo = MediaFileRepository::new(&self.db);

        match repo.find_by_id(file_id).await {
            Ok(Some(file)) => {
                let path = std::path::Path::new(&file.file_path);
                if path.exists() {
                    let data = tokio::fs::read(path).await?;
                    let mime_type = file.mime_type.unwrap_or_else(|| {
                        guess_mime_type(&file.file_name)
                    });
                    Ok(Some((data, mime_type)))
                } else {
                    Ok(None)
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }
}

/// Guess MIME type from file extension
fn guess_mime_type(file_name: &str) -> String {
    let ext = file_name
        .rsplit('.')
        .next()
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "png" => "image/png".to_string(),
        "gif" => "image/gif".to_string(),
        "webp" => "image/webp".to_string(),
        "heic" | "heif" => "image/heic".to_string(),
        "mp4" => "video/mp4".to_string(),
        "mov" => "video/quicktime".to_string(),
        "avi" => "video/x-msvideo".to_string(),
        "mkv" => "video/x-matroska".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

/// Check if a file format is natively supported by browsers (can be served directly without transcoding)
/// Browser-native formats: JPEG, PNG, GIF, WebP, AVIF, SVG
/// Formats that need transcoding: HEIC/HEIF, TIFF, BMP
fn is_browser_native_format(file_name: &str) -> bool {
    file_name
        .rsplit('.')
        .next()
        .map(|s| {
            let ext = s.to_lowercase();
            matches!(
                ext.as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "avif" | "svg"
            )
        })
        .unwrap_or(false)
}

/// Guess MIME type from file path or ID (used for cache lookup)
fn guess_mime_type_from_path(file_name: &str) -> String {
    let ext = file_name
        .rsplit('.')
        .next()
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "png" => "image/png".to_string(),
        "gif" => "image/gif".to_string(),
        "webp" => "image/webp".to_string(),
        "avif" => "image/avif".to_string(),
        "svg" => "image/svg+xml".to_string(),
        "heic" | "heif" => "image/heic".to_string(),
        "tiff" | "tif" => "image/tiff".to_string(),
        "bmp" => "image/bmp".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}
