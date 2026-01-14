use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

/// Media type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Image,
    Video,
    Heif,
}

/// Media metadata extracted from a file
#[derive(Debug, Default)]
pub struct MediaMetadata {
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub exif_timestamp: Option<NaiveDateTime>,
    pub exif_timezone_offset: Option<String>,
    pub create_time: Option<NaiveDateTime>,
    pub modify_time: Option<NaiveDateTime>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub exposure_time: Option<String>,
    pub aperture: Option<String>,
    pub iso: Option<i32>,
    pub focal_length: Option<String>,
    pub duration: Option<f64>,
    pub video_codec: Option<String>,
}

/// Processing error
#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Processing error: {0}")]
    Processing(String),

    #[error("External tool error: {0}")]
    ExternalTool(String),
}

impl From<image::ImageError> for ProcessingError {
    fn from(e: image::ImageError) -> Self {
        ProcessingError::Processing(e.to_string())
    }
}

/// Trait for media processors
#[async_trait]
pub trait MediaProcessor: Send + Sync {
    /// Check if this processor supports the given file
    fn supports(&self, path: &Path) -> bool;

    /// Get the priority of this processor (higher = checked first)
    fn priority(&self) -> i32;

    /// Get the media type this processor handles
    fn media_type(&self) -> MediaType;

    /// Process the file and extract metadata
    async fn process(&self, path: &Path) -> Result<MediaMetadata, ProcessingError>;

    /// Generate a thumbnail for the file
    async fn generate_thumbnail(
        &self,
        path: &Path,
        target_width: u32,
        quality: f32,
    ) -> Result<Option<Vec<u8>>, ProcessingError>;
}

/// Registry for managing media processors
#[derive(Default)]
pub struct ProcessorRegistry {
    processors: Vec<Arc<dyn MediaProcessor>>,
}

impl ProcessorRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    /// Register a processor
    pub fn register(&mut self, processor: Arc<dyn MediaProcessor>) {
        self.processors.push(processor);
        // Sort by priority (descending)
        self.processors.sort_by_key(|p| std::cmp::Reverse(p.priority()));
    }

    /// Find the appropriate processor for a file
    pub fn find_processor(&self, path: &Path) -> Option<Arc<dyn MediaProcessor>> {
        self.processors
            .iter()
            .find(|p| p.supports(path))
            .cloned()
    }
}

/// Get image dimensions from file
pub fn get_image_dimensions(path: &Path) -> Result<(u32, u32), ProcessingError> {
    use std::fs::File;
    use std::io::BufReader;
    use image::GenericImageView;

    let file = File::open(path).map_err(|e| ProcessingError::IoError(e))?;
    let reader = BufReader::new(file);
    let decoder = image::io::Reader::new(reader)
        .with_guessed_format()
        .map_err(|e| ProcessingError::Processing(e.to_string()))?;

    if let Some(format) = decoder.format() {
        let dimensions = decoder
            .decode()
            .map_err(|e| ProcessingError::Processing(e.to_string()))?
            .dimensions();
        Ok(dimensions)
    } else {
        Err(ProcessingError::UnsupportedFormat(
            path.extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        ))
    }
}
