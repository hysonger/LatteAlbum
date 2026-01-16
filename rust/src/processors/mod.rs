pub mod processor_trait;
pub mod image_processor;
pub mod heif_processor; // Enabled: uses image crate's built-in HEIF support
pub mod video_processor;
pub mod file_metadata; // Unified file metadata extraction (file_size, create_time, modify_time)

pub use processor_trait::{MediaProcessor, MediaMetadata, MediaType, ProcessingError, ProcessorRegistry};
pub use image_processor::ExifTag;
