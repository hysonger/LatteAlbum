-- Latte Album Database Schema
-- SQLite database for storing media file metadata

-- Media files table
CREATE TABLE IF NOT EXISTS media_files (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    file_type TEXT NOT NULL,          -- 'image' or 'video'
    mime_type TEXT,
    file_size INTEGER,

    -- Dimensions
    width INTEGER,
    height INTEGER,

    -- Time fields
    exif_timestamp TEXT,              -- ISO8601 UTC
    exif_timezone_offset TEXT,
    create_time TEXT,
    modify_time TEXT,
    last_scanned TEXT,

    -- Camera information
    camera_make TEXT,
    camera_model TEXT,
    lens_model TEXT,

    -- Exposure settings
    exposure_time TEXT,
    aperture TEXT,
    iso INTEGER,
    focal_length TEXT,

    -- Video specific
    duration REAL,
    video_codec TEXT,

    -- Processing status
    thumbnail_generated INTEGER DEFAULT 0
);

-- Indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_media_files_file_type ON media_files(file_type);
CREATE INDEX IF NOT EXISTS idx_media_files_camera_model ON media_files(camera_model);
CREATE INDEX IF NOT EXISTS idx_media_files_exif_timestamp ON media_files(exif_timestamp);
CREATE INDEX IF NOT EXISTS idx_media_files_create_time ON media_files(create_time);
CREATE INDEX IF NOT EXISTS idx_media_files_modify_time ON media_files(modify_time);
CREATE INDEX IF NOT EXISTS idx_media_files_path ON media_files(file_path);

-- Directories table
CREATE TABLE IF NOT EXISTS directories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    parent_id INTEGER,
    last_modified TEXT,
    FOREIGN KEY (parent_id) REFERENCES directories(id)
);

CREATE INDEX IF NOT EXISTS idx_directories_path ON directories(path);
CREATE INDEX IF NOT EXISTS idx_directories_parent_id ON directories(parent_id);

-- System configuration table
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at TEXT
);

-- Scan history table
CREATE TABLE IF NOT EXISTS scan_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time TEXT NOT NULL,
    end_time TEXT,
    files_scanned INTEGER DEFAULT 0,
    files_added INTEGER DEFAULT 0,
    files_updated INTEGER DEFAULT 0,
    files_deleted INTEGER DEFAULT 0,
    status TEXT NOT NULL
);
