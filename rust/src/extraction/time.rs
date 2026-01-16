use chrono::{Datelike, NaiveDateTime, Utc};
use thiserror::Error;

/// Time validation errors
#[derive(Debug, Error)]
pub enum TimeValidationError {
    #[error("Year out of valid range (1900-current year+1): {0}")]
    InvalidYear(i32),

    #[error("Future timestamp not allowed: {0}")]
    FutureTimestamp(NaiveDateTime),
}

/// Time utilities for sorting and validation
pub struct TimeUtils;

impl TimeUtils {
    /// Get effective sort time for a media file
    /// Priority: EXIF timestamp > create time > modify time
    pub fn get_effective_sort_time(
        exif_timestamp: Option<NaiveDateTime>,
        create_time: Option<NaiveDateTime>,
        modify_time: Option<NaiveDateTime>,
    ) -> Option<NaiveDateTime> {
        // Priority 1: EXIF timestamp (must be valid)
        if let Some(ts) = exif_timestamp {
            if Self::is_valid_exif_timestamp(&ts) {
                return Some(ts);
            }
        }

        // Priority 2: Create time (cannot be in future)
        if let Some(ct) = create_time {
            if Self::is_valid_create_time(&ct) {
                return Some(ct);
            }
        }

        // Priority 3: Modify time (last resort)
        modify_time
    }

    /// Validate EXIF timestamp
    /// Must be between 1900 and current year + 1
    pub fn is_valid_exif_timestamp(time: &NaiveDateTime) -> bool {
        let year = time.year();
        let current_year = Utc::now().year();
        year >= 1900 && year <= current_year + 1
    }

    /// Validate create time
    /// Cannot be in the future
    pub fn is_valid_create_time(time: &NaiveDateTime) -> bool {
        let now = Utc::now().naive_utc();
        *time <= now
    }

    /// Parse EXIF date string (format: "YYYY:MM:DD HH:MM:SS")
    pub fn parse_exif_datetime(s: &str) -> Option<NaiveDateTime> {
        NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S").ok()
    }

    /// Format timestamp for display
    pub fn format_for_display(time: &NaiveDateTime) -> String {
        time.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Format timestamp for API response (ISO 8601)
    pub fn format_iso8601(time: &NaiveDateTime) -> String {
        time.format("%Y-%m-%dT%H:%M:%S").to_string()
    }

    /// Format just the date for calendar grouping
    pub fn format_date_only(time: &NaiveDateTime) -> String {
        time.format("%Y-%m-%d").to_string()
    }

    /// Parse date filter string (YYYY-MM-DD)
    pub fn parse_date_filter(s: &str) -> Option<String> {
        // Check if it matches YYYY-MM-DD format
        if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
            Some(format!("{}%", s))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_valid_exif_timestamp() {
        let valid_time = NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        assert!(TimeUtils::is_valid_exif_timestamp(&valid_time));

        // Old timestamp (before 1900)
        let old_time = NaiveDate::from_ymd_opt(1800, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert!(!TimeUtils::is_valid_exif_timestamp(&old_time));
    }

    #[test]
    fn test_valid_create_time() {
        let past_time = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        assert!(TimeUtils::is_valid_create_time(&past_time));
    }

    #[test]
    fn test_get_effective_sort_time() {
        let exif = NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let create = NaiveDate::from_ymd_opt(2024, 6, 16)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let modify = NaiveDate::from_ymd_opt(2024, 6, 17)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        // EXIF has priority
        let result = TimeUtils::get_effective_sort_time(Some(exif), Some(create), Some(modify));
        assert_eq!(result, Some(exif));

        // Without EXIF, use create
        let result = TimeUtils::get_effective_sort_time(None, Some(create), Some(modify));
        assert_eq!(result, Some(create));

        // Without EXIF and create, use modify
        let result = TimeUtils::get_effective_sort_time(None, None, Some(modify));
        assert_eq!(result, Some(modify));

        // None available
        let result = TimeUtils::get_effective_sort_time(None, None, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_exif_datetime() {
        let result = TimeUtils::parse_exif_datetime("2024:06:15 12:30:45");
        assert!(result.is_some());

        let result = TimeUtils::parse_exif_datetime("invalid");
        assert!(result.is_none());
    }

    #[test]
    fn test_format_functions() {
        let time = NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(12, 30, 45)
            .unwrap();

        assert_eq!(TimeUtils::format_for_display(&time), "2024-06-15 12:30:45");
        assert_eq!(TimeUtils::format_iso8601(&time), "2024-06-15T12:30:45");
        assert_eq!(TimeUtils::format_date_only(&time), "2024-06-15");
    }

    #[test]
    fn test_parse_date_filter() {
        assert_eq!(TimeUtils::parse_date_filter("2024-06-15"), Some("2024-06-15%".to_string()));
        assert_eq!(TimeUtils::parse_date_filter("invalid"), None);
        assert_eq!(TimeUtils::parse_date_filter("2024-6-15"), None);
    }
}
