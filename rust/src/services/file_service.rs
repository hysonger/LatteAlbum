use crate::db::{DatabasePool, MediaFileRepository};
use crate::services::CacheService;
use std::path::Path;
use std::sync::Arc;

/// Service for file operations
pub struct FileService {
    db: DatabasePool,
    cache: Arc<CacheService>,
}

impl FileService {
    pub fn new(db: DatabasePool, cache: Arc<CacheService>) -> Self {
        Self { db, cache }
    }

    /// Get thumbnail for a file
    pub async fn get_thumbnail(
        &self,
        file_id: &str,
        target_width: u32,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        // Try cache first
        let size_label = match target_width {
            w if w <= 300 => "small",
            w if w <= 450 => "medium",
            _ => "large",
        };

        if let Some(data) = self.cache.get_thumbnail(file_id, size_label).await {
            return Ok(Some(data));
        }

        // Not in cache, need to generate
        // This would call the processor to generate the thumbnail
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
