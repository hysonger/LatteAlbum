//! Unified file metadata extraction for all media types.
//! Handles file_size, create_time, and modify_time which are format-independent.

use crate::processors::processor_trait::MediaMetadata;
use std::path::Path;

/// Extract file metadata that is common to all file types.
/// This includes file size, creation time, and modification time.
pub fn extract_file_metadata(path: &Path) -> MediaMetadata {
    let mut metadata = MediaMetadata::default();

    if let Ok(file_meta) = path.metadata() {
        metadata.file_size = Some(file_meta.len() as i64);

        metadata.create_time = file_meta
            .created()
            .ok()
            .and_then(system_time_to_naive_datetime);

        metadata.modify_time = file_meta
            .modified()
            .ok()
            .and_then(system_time_to_naive_datetime);
    }

    metadata
}

/// Convert std::time::SystemTime to chrono::NaiveDateTime
fn system_time_to_naive_datetime(time: std::time::SystemTime) -> Option<chrono::NaiveDateTime> {
    let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
    chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
        .map(|dt| dt.naive_utc())
}
