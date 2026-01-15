# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Latte Album is a personal photo album application for NAS deployment, rewritten from Java to Rust. The backend is now built with Rust (Axum, SQLx, SQLite) while the frontend remains Vue 3 + TypeScript + Element Plus.

## Build Commands

### Rust Backend (in `rust/` directory)

```bash
cargo run                              # Development
cargo build --release                  # Release build
cargo test                             # All tests
cargo test test_name                   # Specific test
cargo test -- --nocapture              # With output
cargo check                            # Check without linking
cargo fmt                              # Format code
cargo clippy                           # Lint
```

### Frontend (in `frontend/` directory)

```bash
npm install                            # Install dependencies
npm run dev                            # Dev server (port 5173)
npm run build                          # Production build (type-checked)
npm run preview                        # Preview production build
```

## Environment Variables

Configure backend via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `LATTE_HOST` | `0.0.0.0` | Server bind address |
| `LATTE_PORT` | `8080` | Server port |
| `LATTE_BASE_PATH` | `./photos` | Photo directory |
| `LATTE_DB_PATH` | `./data/album.db` | SQLite database path |
| `LATTE_CACHE_DIR` | `./cache` | Thumbnail cache directory |
| `LATTE_STATIC_DIR` | `./static/dist` | Frontend static files |
| `LATTE_THUMBNAIL_SMALL` | `300` | Small thumbnail width (px) |
| `LATTE_THUMBNAIL_MEDIUM` | `450` | Medium thumbnail width (px) |
| `LATTE_THUMBNAIL_LARGE` | `900` | Large thumbnail width (px) |
| `LATTE_THUMBNAIL_QUALITY` | `0.8` | JPEG quality (80%) |
| `LATTE_SCAN_CRON` | `0 0 2 * * ?` | Scheduled scan cron (2 AM daily) |
| `LATTE_VIDEO_FFMPEG_PATH` | `/usr/bin/ffmpeg` | FFmpeg executable path |

## Architecture

### Backend Structure (Rust)

```
rust/src/
├── main.rs              # Application entry point
├── lib.rs               # Module exports
├── app.rs               # App struct and router configuration
├── config.rs            # Configuration loading from env vars
├── api/                 # REST API handlers
│   ├── files.rs         # File operations, thumbnails, neighbors
│   ├── directories.rs   # Directory tree
│   └── system.rs        # Rescan, status, progress
├── db/                  # Database layer
│   ├── pool.rs          # sqlx connection pool
│   ├── models.rs        # MediaFile, Directory, DateInfo
│   └── repository.rs    # Data access layer
├── services/            # Business logic
│   ├── scan_service.rs  # File scanning and metadata extraction
│   ├── file_service.rs  # File serving and thumbnail generation
│   ├── cache_service.rs # Moka-based cache management
│   └── scheduler.rs     # tokio-cron-scheduler for scheduled scans
├── processors/          # Media format handlers (plugin architecture)
│   ├── processor_trait.rs  # MediaProcessor trait
│   ├── image_processor.rs  # JPEG/PNG/GIF/WebP/TIFF
│   ├── heif_processor.rs   # HEIC/HEIF support via libheif-rs
│   ├── video_processor.rs  # Video thumbnails via FFmpeg
│   └── file_metadata.rs    # Unified file metadata extraction
├── extraction/          # Metadata utilities
│   └── time.rs          # EXIF timestamp extraction
├── websocket/           # WebSocket handlers for scan progress
└── utils/               # Helper functions
```

### Key Services

| Service | Responsibility |
|---------|---------------|
| `FileService` | Thumbnail generation, original file serving |
| `ScanService` | File scanning, metadata extraction |
| `CacheService` | Moka-based thumbnail caching |
| `Scheduler` | Daily 2 AM scheduled scan |

### Media Processors (Strategy Pattern)

| Processor | Extensions | Priority |
|-----------|------------|----------|
| `HeifImageProcessor` | .heic, .heif | 100 |
| `StandardImageProcessor` | .jpg, .jpeg, .png, .gif, .bmp, .webp, .tiff | 10 |
| `VideoProcessor` | .mp4, .avi, .mov, .mkv, .wmv, .flv, .webm | 10 |

Processors are registered in `app.rs` via `ProcessorRegistry`. Higher priority matches first.

### API Endpoints

**File Operations**:
- `GET /api/files` - List with pagination, sorting, filtering
- `GET /api/files/dates` - Get dates with photos
- `GET /api/files/{id}` - File details
- `GET /api/files/{id}/thumbnail?size={small|medium|large|full}` - Thumbnail stream
- `GET /api/files/{id}/original` - Original file stream (download only)
- `GET /api/files/{id}/neighbors` - Prev/next for navigation
- `GET /api/directories` - Directory tree

**System Operations**:
- `POST /api/system/rescan` - Trigger directory rescan
- `GET /api/system/status` - System status
- `GET /api/system/scan/progress` - Scan progress (HTTP fallback)
- `WS /ws/scan` - WebSocket for real-time scan progress

### Frontend Structure

```
frontend/src/
├── views/
│   └── HomeView.vue     # Main page with Gallery + PhotoViewer
├── components/
│   ├── Gallery.vue      # Masonry layout gallery
│   ├── PhotoViewer.vue  # Fullscreen viewer
│   ├── MediaCard.vue    # Thumbnail card
│   └── DateNavigator.vue # Calendar date picker
├── stores/
│   └── gallery.ts       # Pinia store for gallery state
├── services/
│   ├── api.ts           # Axios API client
│   └── websocket.ts     # WebSocket handler
└── types/
    └── index.ts         # TypeScript interfaces
```

### Thumbnail Size System

| Size | Width | Use Case |
|------|-------|----------|
| `small` | 300px | Grid view thumbnails |
| `medium` | 450px | Default |
| `large` | 900px | High-quality preview |
| `full` | 0 (original) | Full-size transcoded for photo viewer |

**Full size design**: Returns full-resolution JPEG transcoded from original (saves bandwidth for HEIC). Not cached.

## Dependencies

| Layer | Technology |
|-------|------------|
| Web | Axum 0.8, Tower HTTP, Tokio |
| Database | SQLx 0.8 with SQLite |
| Image | image crate 0.25, libheif-rs 2.5 |
| Video | ffmpeg-next (optional, feature `video-processing`) |
| Cache | Moka 0.12 |
| Scheduling | tokio-cron-scheduler |
| EXIF | kamadak-exif |

## Feature Flags

- `video-processing`: Enable FFmpeg video thumbnail support (requires ffmpeg-next)

## Key Design Patterns

### AppState
Shared application state passed via Axum's `State` extractor:
```rust
pub struct AppState {
    pub config: Config,
    pub db: DatabasePool,
    pub file_service: Arc<FileService>,
    pub scan_service: Arc<ScanService>,
    pub cache_service: Arc<CacheService>,
    pub broadcaster: Arc<ScanProgressBroadcaster>,
    pub processors: Arc<ProcessorRegistry>,
}
```

### Processor Registry
`ProcessorRegistry` manages media processors via plugin architecture. Processors are registered with priority, and the registry finds the appropriate processor for each file.

### Async Trait Pattern
Media processors use `async-trait` crate to enable async methods in trait objects.

## Database

SQLite with sqlx. Migrations in `src/db/migrations/` applied automatically. Key models:
- `MediaFile`: Photo/video file metadata
- `Directory`: Folder information
- `DateInfo`: Date grouping for timeline

## Project Rules

1. **Code Structure**: Keep code simple and clear. Avoid over-abstraction.
2. **Single Responsibility**: Each component/class should have a single, clear responsibility.
3. **Code Reuse**: Consolidate duplicate code. DRY principle.
4. **Comments**: Add clear comments for complex key logic.
5. **Testing**: Ensure unit test coverage for functional additions.
6. **Cleanup**: Remove obsolete code promptly during refactoring.
7. **Privacy**: Don't access production databases or image APIs without explicit authorization.

## Common Tasks

### Add new file format support
1. Create new processor implementing `MediaProcessor` trait
2. Define supported extensions in `supports()` method
3. Set appropriate `priority()` (higher = matches first)
4. Register in `app.rs`

### Add new EXIF field
1. Add field to `MediaMetadata` in `processor_trait.rs`
2. Extract in appropriate processor's `process()` method
3. Add to TypeScript interface in frontend
4. Update frontend display components

### Modify thumbnail sizes
1. Update `LATTE_THUMBNAIL_*` environment variables
2. `get_thumbnail_size()` in `config.rs` handles size mapping
