use crate::db::{DatabasePool, MediaFileRepository};
use crate::processors::ProcessorRegistry;
use crate::services::CacheService;
use std::sync::Arc;
use tracing::debug;

/// Service for file operations
pub struct FileService {
    db: DatabasePool,
    cache: Arc<CacheService>,
    processors: Arc<ProcessorRegistry>,
}

impl FileService {
    pub fn new(db: DatabasePool, cache: Arc<CacheService>, processors: Arc<ProcessorRegistry>) -> Self {
        Self { db, cache, processors }
    }

    /// Get thumbnail for a file
    /// For "full" size (target_width == 0), returns full-size transcoded image without caching
    pub async fn get_thumbnail(
        &self,
        file_id: &str,
        target_width: u32,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        // Check if this is a full-size request
        let is_full_size = target_width == 0;

        // For full-size requests, skip cache (images are too large)
        // Also skip size_label calculation since we're not caching
        if !is_full_size {
            let size_label = match target_width {
                w if w <= 300 => "small",
                w if w <= 450 => "medium",
                _ => "large",
            };

            if let Some(data) = self.cache.get_thumbnail(file_id, size_label).await {
                return Ok(Some(data));
            }
        }

        // Not in cache or full-size request, generate thumbnail
        let repo = MediaFileRepository::new(&self.db);

        match repo.find_by_id(file_id).await {
            Ok(Some(file)) => {
                let path = std::path::Path::new(&file.file_path);
                if path.exists() {
                    // Find appropriate processor
                    if let Some(processor) = self.processors.find_processor(path) {
                        match processor.generate_thumbnail(path, target_width, 0.9).await {
                            Ok(Some(thumbnail_data)) => {
                                // Cache the generated thumbnail (only for non-full sizes)
                                if !is_full_size {
                                    let size_label = match target_width {
                                        w if w <= 300 => "small",
                                        w if w <= 450 => "medium",
                                        _ => "large",
                                    };
                                    let _ = self.cache.put_thumbnail(file_id, size_label, &thumbnail_data).await;
                                }
                                return Ok(Some(thumbnail_data));
                            }
                            Ok(None) => {
                                debug!("Processor returned no thumbnail for {}", file_id);
                            }
                            Err(e) => {
                                debug!("Failed to generate thumbnail for {}: {}", file_id, e);
                            }
                        }
                    } else {
                        debug!("No processor found for file: {}", file_id);
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
        // Skip fallback for full-size requests (they should go through proper processing)
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
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let repo = MediaFileRepository::new(&self.db);

        match repo.find_by_id(file_id).await {
            Ok(Some(file)) => {
                let path = std::path::Path::new(&file.file_path);
                if path.exists() {
                    // For images, try to use the original file directly (scaled)
                    if file.file_type == "image" {
                        let data = tokio::fs::read(path).await?;
                        // Basic JPEG check - if it's not a JPEG, we can't serve it as thumbnail
                        if data.starts_with(&[0xFF, 0xD8]) || data.starts_with(b"\x89PNG\r\n\x1a\n") {
                            return Ok(Some(data));
                        }
                    }
                }
            }
            _ => {}
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
