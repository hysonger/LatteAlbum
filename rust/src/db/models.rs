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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_media_file_new() {
        let file = MediaFile::new(
            "/photos/vacation.jpg".to_string(),
            "vacation.jpg".to_string(),
            "image".to_string(),
        );

        assert_eq!(file.file_path, "/photos/vacation.jpg");
        assert_eq!(file.file_name, "vacation.jpg");
        assert_eq!(file.file_type, "image");
        assert!(!file.id.is_empty());
        assert!(!file.thumbnail_generated);
        assert!(file.mime_type.is_none());
        assert!(file.width.is_none());
        assert!(file.height.is_none());
    }

    #[test]
    fn test_media_file_get_effective_sort_time_with_exif() {
        let exif_time = NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap();
        let create_time = NaiveDate::from_ymd_opt(2024, 6, 16)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let modify_time = NaiveDate::from_ymd_opt(2024, 6, 17)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let mut file = MediaFile::new("/test.jpg".to_string(), "test.jpg".to_string(), "image".to_string());
        file.exif_timestamp = Some(exif_time);
        file.create_time = Some(create_time);
        file.modify_time = Some(modify_time);

        let result = file.get_effective_sort_time();
        assert_eq!(result, Some(exif_time));
    }

    #[test]
    fn test_media_file_get_effective_sort_time_without_exif() {
        let create_time = NaiveDate::from_ymd_opt(2024, 6, 16)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let modify_time = NaiveDate::from_ymd_opt(2024, 6, 17)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let mut file = MediaFile::new("/test.jpg".to_string(), "test.jpg".to_string(), "image".to_string());
        file.exif_timestamp = None;
        file.create_time = Some(create_time);
        file.modify_time = Some(modify_time);

        let result = file.get_effective_sort_time();
        assert_eq!(result, Some(create_time));
    }

    #[test]
    fn test_media_file_get_effective_sort_time_only_modify() {
        let modify_time = NaiveDate::from_ymd_opt(2024, 6, 17)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let mut file = MediaFile::new("/test.jpg".to_string(), "test.jpg".to_string(), "image".to_string());
        file.exif_timestamp = None;
        file.create_time = None;
        file.modify_time = Some(modify_time);

        let result = file.get_effective_sort_time();
        assert_eq!(result, Some(modify_time));
    }

    #[test]
    fn test_media_file_get_effective_sort_time_none() {
        let file = MediaFile::new("/test.jpg".to_string(), "test.jpg".to_string(), "image".to_string());

        let result = file.get_effective_sort_time();
        assert!(result.is_none());
    }

    #[test]
    fn test_media_file_get_effective_sort_time_invalid_exif() {
        let old_exif = NaiveDate::from_ymd_opt(1800, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let create_time = NaiveDate::from_ymd_opt(2024, 6, 16)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let mut file = MediaFile::new("/test.jpg".to_string(), "test.jpg".to_string(), "image".to_string());
        file.exif_timestamp = Some(old_exif);
        file.create_time = Some(create_time);
        file.modify_time = None;

        let result = file.get_effective_sort_time();
        assert_eq!(result, Some(create_time));
    }

    #[test]
    fn test_file_type_from_string() {
        assert_eq!(FileType::from("image".to_string()), FileType::Image);
        assert_eq!(FileType::from("video".to_string()), FileType::Video);
        assert_eq!(FileType::from("IMAGE".to_string()), FileType::Image);
        assert_eq!(FileType::from("VIDEO".to_string()), FileType::Video);
        assert_eq!(FileType::from("unknown".to_string()), FileType::Image);
    }

    #[test]
    fn test_file_type_from_str() {
        assert_eq!(FileType::from("image"), FileType::Image);
        assert_eq!(FileType::from("video"), FileType::Video);
        assert_eq!(FileType::from("IMAGE"), FileType::Image);
        assert_eq!(FileType::from("VIDEO"), FileType::Video);
        assert_eq!(FileType::from("unknown"), FileType::Image);
    }

    #[test]
    fn test_date_info_serde() {
        let date_info = DateInfo {
            date: "2024-06-15".to_string(),
            count: 42,
        };

        let json = serde_json::to_string(&date_info).unwrap();
        assert!(json.contains("\"date\":\"2024-06-15\""));
        assert!(json.contains("\"count\":42"));
    }

    #[test]
    fn test_directory_serde() {
        let dir = Directory {
            id: 1,
            path: "/photos".to_string(),
            parent_id: None,
            last_modified: None,
        };

        let json = serde_json::to_string(&dir).unwrap();
        assert!(json.contains("\"path\":\"/photos\""));
    }

    #[test]
    fn test_system_config_serde() {
        let config = SystemConfig {
            key: "test_key".to_string(),
            value: Some("test_value".to_string()),
            updated_at: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"key\":\"test_key\""));
    }

    #[test]
    fn test_scan_history_serde() {
        let history = ScanHistory {
            id: 1,
            start_time: Utc::now(),
            end_time: None,
            files_scanned: 100,
            files_added: 50,
            files_updated: 30,
            files_deleted: 5,
            status: "completed".to_string(),
        };

        let json = serde_json::to_string(&history).unwrap();
        assert!(json.contains("\"status\":\"completed\""));
    }

    #[test]
    fn test_is_valid_exif_time() {
        let valid_time = NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        assert!(is_valid_exif_time(&valid_time));

        let old_time = NaiveDate::from_ymd_opt(1800, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert!(!is_valid_exif_time(&old_time));
    }

    #[test]
    fn test_is_valid_create_time() {
        let past_time = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert!(is_valid_create_time(&past_time));
    }

    #[test]
    fn test_media_file_serialization() {
        let mut file = MediaFile::new("/test.jpg".to_string(), "test.jpg".to_string(), "image".to_string());
        file.width = Some(1920);
        file.height = Some(1080);
        file.mime_type = Some("image/jpeg".to_string());

        let json = serde_json::to_string(&file).unwrap();
        assert!(json.contains("\"filePath\":\"/test.jpg\""));
        assert!(json.contains("\"width\":1920"));
        assert!(json.contains("\"height\":1080"));
    }

    #[test]
    fn test_media_file_deserialization() {
        let json = r#"{
            "id": "test-id",
            "filePath": "/test.jpg",
            "fileName": "test.jpg",
            "fileType": "image",
            "mimeType": "image/jpeg",
            "width": 1920,
            "height": 1080,
            "exifTimestamp": null,
            "createTime": null,
            "modifyTime": null,
            "lastScanned": null,
            "cameraMake": null,
            "cameraModel": null,
            "lensModel": null,
            "exposureTime": null,
            "aperture": null,
            "iso": null,
            "focalLength": null,
            "duration": null,
            "videoCodec": null,
            "thumbnailGenerated": false
        }"#;

        let file: MediaFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.id, "test-id");
        assert_eq!(file.file_path, "/test.jpg");
        assert_eq!(file.width, Some(1920));
        assert_eq!(file.height, Some(1080));
    }
}
