-- Create directories table
CREATE TABLE IF NOT EXISTS directories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    parent_path TEXT,
    name TEXT NOT NULL,
    is_valid BOOLEAN DEFAULT 1,
    last_scanned DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create index for directory path lookup
CREATE INDEX IF NOT EXISTS idx_directories_path ON directories(path);

-- Create index for parent path lookup
CREATE INDEX IF NOT EXISTS idx_directories_parent_path ON directories(parent_path);
