-- Create media_files table
CREATE TABLE IF NOT EXISTS media_files (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    file_type TEXT NOT NULL,
    mime_type TEXT,
    file_size INTEGER,

    -- Dimensions
    width INTEGER,
    height INTEGER,

    -- Time fields (stored in UTC)
    exif_timestamp DATETIME,
    exif_timezone_offset TEXT,
    create_time DATETIME,
    modify_time DATETIME,
    last_scanned DATETIME,

    -- Camera info
    camera_make TEXT,
    camera_model TEXT,
    lens_model TEXT,

    -- Exposure settings
    aperture TEXT,
    exposure_time TEXT,
    iso INTEGER,
    focal_length TEXT,

    -- Video specific
    duration REAL,
    video_codec TEXT,

    -- Processing
    thumbnail_generated BOOLEAN DEFAULT 0,

    -- Directory reference
    directory_id INTEGER REFERENCES directories(id),

    -- Created/Updated timestamps
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_media_files_file_path ON media_files(file_path);
CREATE INDEX IF NOT EXISTS idx_media_files_file_type ON media_files(file_type);
CREATE INDEX IF NOT EXISTS idx_media_files_exif_timestamp ON media_files(exif_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_media_files_create_time ON media_files(create_time DESC);
CREATE INDEX IF NOT EXISTS idx_media_files_modify_time ON media_files(modify_time DESC);
CREATE INDEX IF NOT EXISTS idx_media_files_directory_id ON media_files(directory_id);
CREATE INDEX IF NOT EXISTS idx_media_files_camera_model ON media_files(camera_model);

-- Create index for date grouping (YYYY-MM-DD)
CREATE INDEX IF NOT EXISTS idx_media_files_date ON media_files(
    CASE WHEN exif_timestamp IS NOT NULL THEN exif_timestamp
         WHEN create_time IS NOT NULL THEN create_time
         ELSE modify_time END
);
