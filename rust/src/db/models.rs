use chrono::{Datelike, DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Custom serialization for NaiveDateTime to ISO string format
mod date_serialization {
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Deserializer};

    pub fn serialize<S>(date: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match date {
            Some(d) => serializer.serialize_str(&d.format("%Y-%m-%dT%H:%M:%S").to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        match s {
            Some(str) => Ok(NaiveDateTime::parse_from_str(&str, "%Y-%m-%dT%H:%M:%S").ok()),
            None => Ok(None),
        }
    }
}

/// File type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileType {
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "video")]
    Video,
}

impl From<String> for FileType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "video" => FileType::Video,
            _ => FileType::Image,
        }
    }
}

impl From<&str> for FileType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "video" => FileType::Video,
            _ => FileType::Image,
        }
    }
}

/// Media file entity
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFile {
    pub id: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileType")]
    pub file_type: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
    pub mime_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "fileSize")]
    pub file_size: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "exifTimestamp",
        serialize_with = "date_serialization::serialize",
        deserialize_with = "date_serialization::deserialize"
    )]
    pub exif_timestamp: Option<NaiveDateTime>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "exifTimezoneOffset")]
    pub exif_timezone_offset: Option<String>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "createTime",
        serialize_with = "date_serialization::serialize",
        deserialize_with = "date_serialization::deserialize"
    )]
    pub create_time: Option<NaiveDateTime>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "modifyTime",
        serialize_with = "date_serialization::serialize",
        deserialize_with = "date_serialization::deserialize"
    )]
    pub modify_time: Option<NaiveDateTime>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "lastScanned",
        serialize_with = "date_serialization::serialize",
        deserialize_with = "date_serialization::deserialize"
    )]
    pub last_scanned: Option<NaiveDateTime>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "cameraMake")]
    pub camera_make: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "cameraModel")]
    pub camera_model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "lensModel")]
    pub lens_model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "exposureTime")]
    pub exposure_time: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub aperture: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub iso: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "focalLength")]
    pub focal_length: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "videoCodec")]
    pub video_codec: Option<String>,

    #[serde(rename = "thumbnailGenerated")]
    pub thumbnail_generated: bool,
}

impl MediaFile {
    /// Create a new media file with basic fields
    pub fn new(file_path: String, file_name: String, file_type: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            file_path,
            file_name,
            file_type,
            mime_type: None,
            file_size: None,
            width: None,
            height: None,
            exif_timestamp: None,
            exif_timezone_offset: None,
            create_time: None,
            modify_time: None,
            last_scanned: None,
            camera_make: None,
            camera_model: None,
            lens_model: None,
            exposure_time: None,
            aperture: None,
            iso: None,
            focal_length: None,
            duration: None,
            video_codec: None,
            thumbnail_generated: false,
        }
    }

    /// Get the effective sort time (EXIF > create > modify)
    pub fn get_effective_sort_time(&self) -> Option<NaiveDateTime> {
        // Priority: exif_timestamp > create_time > modify_time
        if let Some(ts) = self.exif_timestamp {
            if is_valid_exif_time(&ts) {
                return Some(ts);
            }
        }
        if let Some(ct) = self.create_time {
            if is_valid_create_time(&ct) {
                return Some(ct);
            }
        }
        self.modify_time
    }
}

/// Directory entity
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Directory {
    pub id: i64,
    pub path: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<NaiveDateTime>,
}

/// Date info for calendar display
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DateInfo {
    pub date: String,           // YYYY-MM-DD format
    pub count:i64,
}

/// System configuration
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SystemConfig {
    pub key: String,
    pub value: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Scan history record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ScanHistory {
    pub id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub files_scanned: i64,
    pub files_added: i64,
    pub files_updated: i64,
    pub files_deleted: i64,
    pub status: String,
}

/// Validates EXIF timestamp (must be between 1900 and current year + 1)
fn is_valid_exif_time(time: &NaiveDateTime) -> bool {
    let year = time.year();
    let current_year = Utc::now().year();
    year >= 1900 && year <= current_year + 1
}

/// Validates create time (cannot be in the future)
fn is_valid_create_time(time: &NaiveDateTime) -> bool {
    let now = Utc::now().naive_utc();
    *time <= now
}
