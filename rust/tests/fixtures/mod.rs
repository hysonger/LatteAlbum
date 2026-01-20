//! Test fixtures for integration tests
//!
//! Provides utilities for creating temporary test directories and test data.

use std::path::PathBuf;
use tempfile::{TempDir, Builder};
use chrono::{NaiveDateTime, Utc, TimeZone};
use uuid::Uuid;

/// Test fixtures manager
pub struct TestFixtures {
    _temp_dir: TempDir,
    test_photos_dir: PathBuf,
}

impl TestFixtures {
    /// Create a new test fixtures instance with a temporary directory
    pub fn new() -> (Self, PathBuf) {
        let temp_dir = Builder::new()
            .prefix("latte_test_")
            .tempdir()
            .expect("Failed to create temp directory");

        let test_photos_dir = temp_dir.path().join("photos");
        std::fs::create_dir_all(&test_photos_dir)
            .expect("Failed to create test photos directory");

        let fixtures = Self {
            _temp_dir: temp_dir,
            test_photos_dir: test_photos_dir.clone(),
        };

        (fixtures, test_photos_dir)
    }

    /// Get the test photos directory
    pub fn photos_dir(&self) -> &PathBuf {
        &self.test_photos_dir
    }

    /// Copy a sample image from source to destination in the test directory
    pub fn copy_sample_image(&self, src: &str, dst: &str) -> PathBuf {
        let dst_path = self.test_photos_dir.join(dst);
        std::fs::copy(src, &dst_path)
            .expect(&format!("Failed to copy test image from {} to {}", src, dst_path.display()));
        dst_path
    }

    /// Create an empty subdirectory for testing
    pub fn create_subdir(&self, name: &str) -> PathBuf {
        let subdir = self.test_photos_dir.join(name);
        std::fs::create_dir_all(&subdir)
            .expect(&format!("Failed to create subdirectory: {}", name));
        subdir
    }
}

/// Create a test media file with default values
pub fn create_test_media_file(file_name: &str) -> latte_album::db::MediaFile {
    // Use chrono::DateTime to create a timestamp and convert to NaiveDateTime
    let timestamp = Utc.timestamp_opt(1700000000, 0).unwrap();

    latte_album::db::MediaFile {
        id: Uuid::new_v4().to_string(),
        file_path: format!("/test/photos/{}", file_name),
        file_name: file_name.to_string(),
        file_type: "image".to_string(),
        mime_type: Some("image/jpeg".to_string()),
        file_size: Some(1024),
        width: Some(1920),
        height: Some(1080),
        exif_timestamp: Some(timestamp.naive_utc()),
        exif_timezone_offset: Some("+08:00".to_string()),
        create_time: Some(timestamp.naive_utc()),
        modify_time: Some(timestamp.naive_utc()),
        last_scanned: Some(Utc::now().naive_utc()),
        camera_make: Some("TestCamera".to_string()),
        camera_model: Some("TestModel".to_string()),
        lens_model: Some("TestLens".to_string()),
        exposure_time: Some("1/125".to_string()),
        aperture: Some("2.8".to_string()),
        iso: Some(100),
        focal_length: Some("50mm".to_string()),
        duration: None,
        video_codec: None,
        thumbnail_generated: false,
    }
}

/// Create a test media file with custom values
pub fn create_test_media_file_with(
    file_name: &str,
    file_type: &str,
    exif_timestamp: Option<NaiveDateTime>,
) -> latte_album::db::MediaFile {
    // Use chrono::DateTime to create a timestamp
    let default_timestamp = Utc.timestamp_opt(1700000000, 0).unwrap();
    let timestamp = exif_timestamp.unwrap_or_else(|| default_timestamp.naive_utc());

    latte_album::db::MediaFile {
        id: Uuid::new_v4().to_string(),
        file_path: format!("/test/photos/{}", file_name),
        file_name: file_name.to_string(),
        file_type: file_type.to_string(),
        mime_type: Some(match file_type {
            "image" => "image/jpeg".to_string(),
            "video" => "video/mp4".to_string(),
            _ => "application/octet-stream".to_string(),
        }),
        file_size: Some(1024),
        width: Some(1920),
        height: Some(1080),
        exif_timestamp: Some(timestamp),
        exif_timezone_offset: Some("+08:00".to_string()),
        create_time: Some(timestamp),
        modify_time: Some(timestamp),
        last_scanned: Some(Utc::now().naive_utc()),
        camera_make: Some("TestCamera".to_string()),
        camera_model: Some("TestModel".to_string()),
        lens_model: Some("TestLens".to_string()),
        exposure_time: Some("1/125".to_string()),
        aperture: Some("2.8".to_string()),
        iso: Some(100),
        focal_length: Some("50mm".to_string()),
        duration: if file_type == "video" { Some(10.0) } else { None },
        video_codec: if file_type == "video" { Some("H264".to_string()) } else { None },
        thumbnail_generated: false,
    }
}
