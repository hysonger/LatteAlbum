use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid value for {0}: {1}")]
    InvalidValue(String, String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Config {
    // === Server Configuration ===
    /// Server bind address (default: "0.0.0.0")
    pub host: String,
    /// Server port (default: 8080)
    pub port: u16,

    // === Path Configuration ===
    /// Base directory for photo/video files
    pub base_path: PathBuf,
    /// SQLite database file path
    pub db_path: PathBuf,
    /// Thumbnail cache directory
    pub cache_dir: PathBuf,
    /// Frontend static files directory
    pub static_dir: PathBuf,

    // === Thumbnail Configuration ===
    /// Small thumbnail width in pixels (default: 300)
    pub thumbnail_small: u32,
    /// Medium thumbnail width in pixels (default: 450)
    pub thumbnail_medium: u32,
    /// Large thumbnail height in pixels (default: 900) - fixed height, maintains aspect ratio
    pub thumbnail_large: u32,
    /// JPEG encoding quality 0.0-1.0 (default: 0.8 = 80%)
    pub thumbnail_quality: f32,

    // === Scan Configuration ===
    /// Override for scan worker count (CPU cores * 2 if None)
    pub scan_worker_count: Option<usize>,
    /// Cron expression for scheduled scans (default: "0 0 2 * * ?" = 2 AM daily)
    pub scan_cron: String,
    /// Batch size for database operations during scan (default: 50)
    pub scan_batch_size: usize,

    // === Video Processing Configuration ===
    /// Path to FFmpeg executable
    pub ffmpeg_path: PathBuf,
    /// Video thumbnail capture offset in seconds (default: 1.0)
    pub video_thumbnail_offset: f64,
    /// Video thumbnail capture duration in seconds (default: 0.1)
    pub video_thumbnail_duration: f64,

    // === Cache Configuration ===
    /// Maximum number of items in memory cache (default: 1000)
    pub cache_max_capacity: usize,
    /// Cache time-to-live in seconds (default: 3600 = 1 hour)
    pub cache_ttl_seconds: u64,

    // === Batch Processing Configuration ===
    /// Batch size for checking existing files in database (default: 500)
    pub db_batch_check_size: usize,
    /// Batch size for writing results to database (default: 100)
    pub db_batch_write_size: usize,

    // === WebSocket Configuration ===
    /// Progress broadcast interval - send every N files (default: 10)
    pub ws_progress_broadcast_interval: u64,

    // === API Configuration ===
    /// Default page size for list API responses (default: 50)
    pub api_default_page_size: usize,

    // === Transcoding Pool Configuration ===
    /// Number of threads in Rayon transcoding pool for CPU-intensive image processing (default: 4)
    pub transcoding_threads: usize,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if exists
        dotenvy::dotenv().ok();

        let host = get_env("LATTE_HOST", "0.0.0.0")?;
        let port = get_env_u16("LATTE_PORT", 8080)?;

        let base_path = get_env_path("LATTE_BASE_PATH", "./photos")?;
        let db_path = get_env_path("LATTE_DB_PATH", "./data/album.db")?;
        let cache_dir = get_env_path("LATTE_CACHE_DIR", "./cache")?;
        let static_dir = get_env_path("LATTE_STATIC_DIR", "./static/dist")?;

        let thumbnail_small = get_env_u32("LATTE_THUMBNAIL_SMALL", 300)?;
        let thumbnail_medium = get_env_u32("LATTE_THUMBNAIL_MEDIUM", 600)?;
        let thumbnail_large = get_env_u32("LATTE_THUMBNAIL_LARGE", 900)?;
        let thumbnail_quality = get_env_f32("LATTE_THUMBNAIL_QUALITY", 0.8)?;

        let scan_worker_count = get_env_usize("LATTE_SCAN_WORKER_COUNT", 0)?;
        let scan_worker_count = if scan_worker_count == 0 { None } else { Some(scan_worker_count) };
        let scan_cron = get_env("LATTE_SCAN_CRON", "0 0 2 * * ?")?;
        let scan_batch_size = get_env_usize("LATTE_SCAN_BATCH_SIZE", 50)?;

        let ffmpeg_path = get_env_path("LATTE_VIDEO_FFMPEG_PATH", "/usr/bin/ffmpeg")?;
        let video_thumbnail_offset = get_env_f64("LATTE_VIDEO_THUMBNAIL_OFFSET", 1.0)?;
        let video_thumbnail_duration = get_env_f64("LATTE_VIDEO_THUMBNAIL_DURATION", 0.1)?;

        let cache_max_capacity = get_env_usize("LATTE_CACHE_MAX_CAPACITY", 1000)?;
        let cache_ttl_seconds = get_env_u64("LATTE_CACHE_TTL_SECONDS", 3600)?;

        let db_batch_check_size = get_env_usize("LATTE_DB_BATCH_CHECK_SIZE", 500)?;
        let db_batch_write_size = get_env_usize("LATTE_DB_BATCH_WRITE_SIZE", 100)?;

        let ws_progress_broadcast_interval = get_env_u64("LATTE_WS_PROGRESS_INTERVAL", 10)?;

        let api_default_page_size = get_env_usize("LATTE_API_DEFAULT_PAGE_SIZE", 50)?;

        let transcoding_threads = get_env_usize("LATTE_TRANSCODING_THREADS", 4)?;

        Ok(Self {
            host,
            port,
            base_path,
            db_path,
            cache_dir,
            static_dir,
            thumbnail_small,
            thumbnail_medium,
            thumbnail_large,
            thumbnail_quality,
            scan_worker_count,
            scan_cron,
            scan_batch_size,
            ffmpeg_path,
            video_thumbnail_offset,
            video_thumbnail_duration,
            cache_max_capacity,
            cache_ttl_seconds,
            db_batch_check_size,
            db_batch_write_size,
            ws_progress_broadcast_interval,
            api_default_page_size,
            transcoding_threads,
        })
    }

    /// Get thumbnail size dimension
    /// Returns 0 for "full" size to indicate no resizing (full-size transcoded output)
    pub fn get_thumbnail_size(&self, size: &str) -> u32 {
        match size {
            "small" => self.thumbnail_small,
            "medium" => self.thumbnail_medium,
            "large" => self.thumbnail_large,
            "full" => 0, // 0 indicates full-size transcoded output (no resizing)
            _ => self.thumbnail_medium,
        }
    }
}

fn get_env(key: &str, default: &str) -> Result<String, ConfigError> {
    std::env::var(key).map_or(Ok(default.to_string()), |v| {
        if v.is_empty() {
            Ok(default.to_string())
        } else {
            Ok(v)
        }
    })
}

fn get_env_path(key: &str, default: &str) -> Result<PathBuf, ConfigError> {
    let value = get_env(key, default)?;
    PathBuf::from_str(&value).map_err(|e| ConfigError::InvalidValue(key.to_string(), e.to_string()))
}

fn get_env_u16(key: &str, default: u16) -> Result<u16, ConfigError> {
    let value = get_env(key, "")?;
    if value.is_empty() {
        return Ok(default);
    }
    value.parse().map_or(Ok(default), |v| {
        if v == 0 { Ok(default) } else { Ok(v) }
    })
}

fn get_env_u32(key: &str, default: u32) -> Result<u32, ConfigError> {
    let value = get_env(key, "")?;
    if value.is_empty() {
        return Ok(default);
    }
    value.parse().map_or(Ok(default), |v| {
        if v == 0 { Ok(default) } else { Ok(v) }
    })
}

fn get_env_usize(key: &str, default: usize) -> Result<usize, ConfigError> {
    let value = get_env(key, "")?;
    if value.is_empty() {
        return Ok(default);
    }
    value.parse().map_or(Ok(default), |v| {
        if v == 0 { Ok(default) } else { Ok(v) }
    })
}

fn get_env_u64(key: &str, default: u64) -> Result<u64, ConfigError> {
    let value = get_env(key, "")?;
    if value.is_empty() {
        return Ok(default);
    }
    value.parse().map_or(Ok(default), |v| {
        if v == 0 { Ok(default) } else { Ok(v) }
    })
}

fn get_env_f32(key: &str, default: f32) -> Result<f32, ConfigError> {
    let value = get_env(key, "")?;
    if value.is_empty() {
        return Ok(default);
    }
    value.parse().map_or(Ok(default), |v| {
        if v <= 0.0 || v > 1.0 { Ok(default) } else { Ok(v) }
    })
}

fn get_env_f64(key: &str, default: f64) -> Result<f64, ConfigError> {
    let value = get_env(key, "")?;
    if value.is_empty() {
        return Ok(default);
    }
    value.parse().map_or(Ok(default), |v| {
        if v < 0.0 { Ok(default) } else { Ok(v) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn clear_env_vars() {
        env::remove_var("LATTE_HOST");
        env::remove_var("LATTE_PORT");
        env::remove_var("LATTE_BASE_PATH");
        env::remove_var("LATTE_DB_PATH");
        env::remove_var("LATTE_CACHE_DIR");
        env::remove_var("LATTE_STATIC_DIR");
        env::remove_var("LATTE_THUMBNAIL_SMALL");
        env::remove_var("LATTE_THUMBNAIL_MEDIUM");
        env::remove_var("LATTE_THUMBNAIL_LARGE");
        env::remove_var("LATTE_THUMBNAIL_QUALITY");
        env::remove_var("LATTE_SCAN_CRON");
        env::remove_var("LATTE_VIDEO_FFMPEG_PATH");
        env::remove_var("LATTE_CACHE_MAX_CAPACITY");
        env::remove_var("LATTE_CACHE_TTL_SECONDS");
        env::remove_var("LATTE_WS_PROGRESS_INTERVAL");
        env::remove_var("LATTE_API_DEFAULT_PAGE_SIZE");
    }

    #[test]
    fn test_default_values() {
        clear_env_vars();

        assert_eq!(Config::default().host, "0.0.0.0");
        assert_eq!(Config::default().port, 8080);
        assert_eq!(Config::default().thumbnail_small, 300);
        assert_eq!(Config::default().thumbnail_medium, 600);
        assert_eq!(Config::default().thumbnail_large, 900);
    }

    #[test]
    fn test_get_thumbnail_size() {
        let config = Config {
            thumbnail_small: 300,
            thumbnail_medium: 600,
            thumbnail_large: 900,
            ..Default::default()
        };

        assert_eq!(config.get_thumbnail_size("small"), 300);
        assert_eq!(config.get_thumbnail_size("medium"), 600);
        assert_eq!(config.get_thumbnail_size("large"), 900);
        assert_eq!(config.get_thumbnail_size("full"), 0);
        assert_eq!(config.get_thumbnail_size("unknown"), 600);
    }

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::MissingEnvVar("TEST_VAR".to_string());
        assert_eq!(format!("{}", error), "Missing environment variable: TEST_VAR");

        let error = ConfigError::InvalidValue("TEST_VAR".to_string(), "invalid".to_string());
        assert_eq!(format!("{}", error), "Invalid value for TEST_VAR: invalid");
    }

    #[test]
    fn test_config_full_default() {
        let config = Config::default();

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.base_path, PathBuf::from("./photos"));
        assert_eq!(config.db_path, PathBuf::from("./data/album.db"));
        assert_eq!(config.cache_dir, PathBuf::from("./cache"));
        assert_eq!(config.static_dir, PathBuf::from("./static/dist"));
        assert_eq!(config.thumbnail_small, 300);
        assert_eq!(config.thumbnail_medium, 600);
        assert_eq!(config.thumbnail_large, 900);
        assert_eq!(config.thumbnail_quality, 0.8);
        assert_eq!(config.scan_worker_count, None);
        assert_eq!(config.scan_cron, "0 0 2 * * ?");
        assert_eq!(config.scan_batch_size, 50);
        assert_eq!(config.ffmpeg_path, PathBuf::from("/usr/bin/ffmpeg"));
        assert_eq!(config.video_thumbnail_offset, 1.0);
        assert_eq!(config.video_thumbnail_duration, 0.1);
        assert_eq!(config.cache_max_capacity, 1000);
        assert_eq!(config.cache_ttl_seconds, 3600);
        assert_eq!(config.db_batch_check_size, 500);
        assert_eq!(config.db_batch_write_size, 100);
        assert_eq!(config.ws_progress_broadcast_interval, 10);
        assert_eq!(config.api_default_page_size, 50);
        assert_eq!(config.transcoding_threads, 4);
    }

    #[test]
    fn test_transcoding_threads_config() {
        clear_env_vars();
        std::env::set_var("LATTE_TRANSCODING_THREADS", "8");

        // Reload config
        let config = Config::from_env().unwrap();
        assert_eq!(config.transcoding_threads, 8);

        std::env::remove_var("LATTE_TRANSCODING_THREADS");
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            base_path: PathBuf::from("./photos"),
            db_path: PathBuf::from("./data/album.db"),
            cache_dir: PathBuf::from("./cache"),
            static_dir: PathBuf::from("./static/dist"),
            thumbnail_small: 300,
            thumbnail_medium: 600,
            thumbnail_large: 900,
            thumbnail_quality: 0.8,
            scan_worker_count: None,
            scan_cron: "0 0 2 * * ?".to_string(),
            scan_batch_size: 50,
            ffmpeg_path: PathBuf::from("/usr/bin/ffmpeg"),
            video_thumbnail_offset: 1.0,
            video_thumbnail_duration: 0.1,
            cache_max_capacity: 1000,
            cache_ttl_seconds: 3600,
            db_batch_check_size: 500,
            db_batch_write_size: 100,
            ws_progress_broadcast_interval: 10,
            api_default_page_size: 50,
            transcoding_threads: 4,
        }
    }
}
