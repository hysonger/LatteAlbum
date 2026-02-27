# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Table of Contents

- [Project Overview](#project-overview)
- [Quick Start](#quick-start)
- [Build Commands](#build-commands)
- [Environment Variables](#environment-variables)
- [Architecture](#architecture)
- [API Endpoints](#api-endpoints)
- [Database](#database)
- [Testing](#testing)
- [CI/CD](#cicd)
- [Dependencies](#dependencies)
- [Feature Flags](#feature-flags)
- [Key Design Patterns](#key-design-patterns)
- [Project Rules](#project-rules)
- [Common Tasks](#common-tasks)
- [Guidelines](#guidelines)
- [Known Issues](#known-issues)

## Project Overview

Latte Album is a personal photo album application for NAS deployment, rewritten from Java to Rust. The backend is built with Rust (Axum, SQLx, SQLite) while the frontend uses Vue 3 + TypeScript + Element Plus.

**Version**: 0.1.0

**License**: MIT

### Key Features

- Responsive masonry gallery layout with dual-level lazy loading
- Support for images (JPEG, PNG, GIF, WebP, TIFF, HEIC/HEIF) and videos (MP4, AVI, MOV, MKV, etc.)
- High-performance parallel file scanning with modification time comparison
- Real-time scan progress via WebSocket
- EXIF metadata extraction (camera info, lens, aperture, ISO, etc.)
- Three-tier thumbnail caching (memory, disk, dynamic generation)
- Scheduled automatic scanning (daily at 2 AM)
- Video thumbnail generation via FFmpeg

### Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Axum 0.8, Rust, Tokio, SQLx, SQLite |
| Frontend | Vue 3, TypeScript, Pinia, Element Plus |
| Image Processing | image crate, libheif-rs |
| Video Processing | FFmpeg (ffmpeg-next) |
| EXIF | kamadak-exif (custom fork) |
| Cache | Moka (memory) + disk cache |
| Communication | REST API + WebSocket |

## Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 18+
- libheif (for HEIF format support)
- FFmpeg (optional, for video thumbnails)

### Development

```bash
# Backend (in rust/ directory)
./cargo-with-vendor.sh run

# Frontend (in frontend/ directory, separate terminal)
cd frontend
npm install
npm run dev
```

### Production Build

```bash
# Build backend
cd rust
./cargo-with-vendor.sh build --release

# Build frontend
cd frontend
npm run build

# Package everything
./package.sh
```

## Build Commands

### Rust Backend (in `rust/` directory)

| Command | Description |
|---------|-------------|
| `cargo run` | Development (requires system libheif) |
| `./cargo-with-vendor.sh run` | Development with vendor-built libheif |
| `cargo build --release` | Release build |
| `./cargo-with-vendor.sh build --release` | Release build with vendor-built libheif |
| `cargo test` | All tests |
| `./cargo-with-vendor.sh test` | Tests with vendor-built libheif |
| `cargo check` | Check without linking |
| `./cargo-with-vendor.sh check` | Check with vendor-built libheif |
| `cargo fmt` / `cargo clippy` | Format / Lint |

#### Building with Self-compiled libheif

This project uses a vendored version of libheif instead of the system library. This is necessary because:
- System library versions may be too old to meet requirements of libheif-rs
- Provides consistent build behavior across different environments

**Usage**: Always use `./cargo-with-vendor.sh` instead of plain `cargo` commands:

```bash
cd rust

# Development
./cargo-with-vendor.sh run
./cargo-with-vendor.sh build

# Release
./cargo-with-vendor.sh build --release

# Testing
./cargo-with-vendor.sh test

# Type checking
./cargo-with-vendor.sh check
```

**How it works**:
1. The script builds libheif from `rust/vendor/libheif` using CMake
2. Installs to `rust/target/vendor-build/install`
3. Sets `PKG_CONFIG_PATH` to point to the built library
4. Runs cargo with `--features vendor-build` flag

**Manual approach** (if needed):
```bash
export PKG_CONFIG_PATH="$PWD/target/vendor-build/install/lib/pkgconfig:$PKG_CONFIG_PATH"
cargo run --features vendor-build
```

### Frontend (in `frontend/` directory)

| Command | Description |
|---------|-------------|
| `npm install` | Install dependencies |
| `npm run dev` | Dev server (port 5173) |
| `npm run build` | Production build (type-checked) |

## Environment Variables

Configure backend via environment variables:

### Server Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `LATTE_HOST` | `0.0.0.0` | Server bind address |
| `LATTE_PORT` | `8080` | Server port |

### Path Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `LATTE_BASE_PATH` | `./photos` | Photo directory |
| `LATTE_DB_PATH` | `./data/album.db` | SQLite database path |
| `LATTE_CACHE_DIR` | `./cache` | Thumbnail cache directory |
| `LATTE_STATIC_DIR` | `./static/dist` | Frontend static files |

### Thumbnail Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `LATTE_THUMBNAIL_SMALL` | `300` | Small thumbnail width (px) |
| `LATTE_THUMBNAIL_MEDIUM` | `600` | Medium thumbnail width (px) |
| `LATTE_THUMBNAIL_LARGE` | `900` | Large thumbnail height (px) - fixed height, maintains aspect ratio |
| `LATTE_THUMBNAIL_QUALITY` | `0.8` | JPEG quality (0.0-1.0, default 80%) |

### Scan Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `LATTE_SCAN_WORKER_COUNT` | (auto) | Override worker count (CPU cores x 2, default) |
| `LATTE_SCAN_CRON` | `0 0 2 * * ?` | Scheduled scan cron (2 AM daily) |
| `LATTE_SCAN_BATCH_SIZE` | `50` | Database batch size for scan operations |

### Transcoding Pool Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `LATTE_TRANSCODING_THREADS` | `4` | Number of threads in Rayon transcoding pool for CPU-intensive image processing |

### Video Processing Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `LATTE_VIDEO_FFMPEG_PATH` | `/usr/bin/ffmpeg` | FFmpeg executable path |
| `LATTE_VIDEO_THUMBNAIL_OFFSET` | `1.0` | Video thumbnail capture offset (seconds) |
| `LATTE_VIDEO_THUMBNAIL_DURATION` | `0.1` | Video thumbnail capture duration (seconds) |

### Cache Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `LATTE_CACHE_MAX_CAPACITY` | `1000` | Memory cache max items |
| `LATTE_CACHE_TTL_SECONDS` | `3600` | Cache TTL (seconds, default 1 hour) |

### Database Batch Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `LATTE_DB_BATCH_CHECK_SIZE` | `500` | Batch size for checking existing files |
| `LATTE_DB_BATCH_WRITE_SIZE` | `100` | Batch size for writing results |

### WebSocket Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `LATTE_WS_PROGRESS_INTERVAL` | `10` | Progress broadcast interval (files between broadcasts) |

### API Configuration
| Variable | Default | Description |
|----------|---------|-------------|
| `LATTE_API_DEFAULT_PAGE_SIZE` | `50` | Default page size for list API |

## Architecture

### Backend Structure (Rust)

```
rust/
├── Cargo.toml            # Rust package manifest
├── cargo-with-vendor.sh # Build script with vendored libheif
├── vendor/
│   └── libheif/         # Vendored libheif source (git submodule)
└── src/
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
    │   ├── repository.rs    # Data access layer
    │   └── migrations/      # SQL migrations
    ├── services/            # Business logic
    │   ├── scan_service.rs  # File scanning and metadata extraction
    │   ├── file_service.rs  # File serving and thumbnail generation
    │   ├── cache_service.rs # Moka-based cache management
    │   ├── scheduler.rs     # tokio-cron-scheduler for scheduled scans
    │   └── transcoding_pool.rs # Rayon-based CPU-intensive image processing pool
    ├── processors/          # Media format handlers (plugin architecture)
    │   ├── processor_trait.rs  # MediaProcessor trait
    │   ├── image_processor.rs  # JPEG/PNG/GIF/WebP/TIFF
    │   ├── heif_processor.rs   # HEIC/HEIF support via libheif-rs
    │   ├── video_processor.rs  # Video thumbnails via FFmpeg
    │   └── file_metadata.rs    # Unified file metadata extraction
    ├── extraction/          # Metadata utilities
    │   └── time.rs          # EXIF timestamp extraction
    ├── websocket/           # WebSocket handlers for scan progress
    │   ├── broadcast.rs     # ScanProgressBroadcaster for WebSocket broadcasting
    │   ├── handler.rs       # WebSocket connection handler
    │   └── scan_state.rs    # ScanStateManager - global scan state management
    ├── helpers/             # Helper functions
    ├── fixtures/            # Test fixtures
    └── utils/               # Utility functions
```

### Key Services

| Service | Responsibility |
|---------|---------------|
| `FileService` | Thumbnail generation, original file serving |
| `ScanService` | File scanning, metadata extraction |
| `ScanStateManager` | Global scan state management and progress broadcasting |
| `CacheService` | Moka-based thumbnail caching |
| `Scheduler` | Daily 2 AM scheduled scan |
| `TranscodingPool` | Rayon-based thread pool for CPU-intensive image processing |

### Media Processors (Strategy Pattern)

| Processor | Extensions | Priority |
|-----------|------------|----------|
| `HeifImageProcessor` | .heic, .heif | 100 |
| `StandardImageProcessor` | .jpg, .jpeg, .png, .gif, .bmp, .webp, .tiff | 10 |
| `VideoProcessor` | .mp4, .avi, .mov, .mkv, .wmv, .flv, .webm | 10 |

Processors are registered in `app.rs` via `ProcessorRegistry`. Higher priority matches first.

### Image Transcoding and EXIF Extraction

#### Thumbnail Generation Pipeline

| Format | Decoder | Scaling | Encoder | Output |
|--------|---------|---------|---------|--------|
| JPEG/PNG/GIF/WebP/TIFF | `image` crate | `thumbnail()` (fast integer) | JPEG (80% quality) | JPEG |
| HEIC/HEIF | `libheif-rs` | `image.scale()` (libheif built-in) | JPEG (80% quality) | JPEG |

**Scaling Modes**:
- `small`/`medium`: Fixed width, height proportional
- `large`: Fixed height, width proportional (for large preview placeholders)

**Performance**:
- HEIC decoding is ~170x faster than JPG (3ms vs 555ms)
- JPEG encoding is ~3-7x faster than WebP
- Thumbnails use `spawn_blocking` in blocking thread pool

#### EXIF Metadata Extraction

**Library**: `kamadak-exif`

**Extracted Fields**:
| Field | EXIF Tag | Description |
|-------|----------|-------------|
| `exif_timestamp` | DateTimeOriginal | Capture time |
| `exif_timezone_offset` | OffsetTimeOriginal | Capture timezone |
| `camera_make` | Make | Camera manufacturer |
| `camera_model` | Model | Camera model |
| `lens_model` | LensModel | Lens model |
| `aperture` | FNumber | Aperture value |
| `exposure_time` | ExposureTime | Shutter speed |
| `iso` | ISOSpeedRatings | ISO sensitivity |
| `focal_length` | FocalLength | Focal length |

**Implementation**: `image_processor.rs:extract_exif()` handles all formats including HEIC.

#### File Streaming

**Video Streaming** (`api/files.rs:get_original`):
- Supports HTTP Range requests (206 Partial Content) for progressive playback
- Supports seek via new Range requests
- Large files (>50MB) use `ReaderStream` for memory-efficient streaming
- Returns `Accept-Ranges: bytes` header

**Thumbnail Streaming** (`api/files.rs:get_thumbnail`):
Three-tier caching strategy:
| Tier | Storage | Response |
|------|---------|----------|
| L1 | Memory cache (Moka) | Direct return |
| L2 | Disk cache | `File::open` + `ReaderStream` (32KB chunks) |
| L3 | Dynamic generation | Write to cache, then return |

#### Cache Performance Optimizations

**HEIC stride optimization**: When stride == width * 4, data is tightly packed and can be used directly. Otherwise, padding is removed row by row.

**Bytes optimization**: Memory cache uses `bytes::Bytes` instead of `Vec<u8>`. Cloning is O(1) via reference counting instead of O(n) data copy.

### API Endpoints

**File Operations**:
- `GET /api/files` - List with pagination, sorting, filtering
- `GET /api/files/dates` - Get dates with photos
- `GET /api/files/{id}` - File details
- `GET /api/files/{id}/thumbnail?size={small|medium|large|full}` - Thumbnail stream
- `GET /api/files/{id}/original` - Original file stream with Range support
- `GET /api/files/{id}/neighbors` - Prev/next for navigation
- `GET /api/directories` - Directory tree

**System Operations**:
- `POST /api/system/rescan` - Trigger directory rescan
- `POST /api/system/scan/cancel` - Cancel ongoing scan
- `GET /api/system/status` - System status
- `GET /api/system/scan/progress` - Scan progress (HTTP fallback)
- `WS /ws/scan` - WebSocket for real-time scan progress

### Frontend Structure

```
frontend/
├── package.json         # NPM dependencies
├── vite.config.ts       # Vite configuration
├── tsconfig.json        # TypeScript configuration
├── index.html           # Entry HTML
└── src/
    ├── main.ts             # Application entry point
    ├── App.vue             # Root component
    ├── router/
    │   └── index.ts        # Vue Router configuration
    ├── views/
    │   └── HomeView.vue    # Main page with Gallery + PhotoViewer
    ├── components/
    │   ├── Gallery.vue         # Masonry layout gallery with lazy loading
    │   ├── PhotoViewer.vue     # Fullscreen viewer with metadata
    │   ├── MediaCard.vue       # Thumbnail card with Intersection Observer
    │   ├── DateNavigator.vue   # Calendar date picker
    │   ├── FilterControls.vue  # File type/date filtering
    │   ├── SortControls.vue    # Sorting options
    │   ├── RefreshButton.vue   # Manual rescan trigger
    │   ├── ScanProgressPopup.vue # Scan progress display
    │   └── MobileMenu.vue      # Mobile navigation
    ├── composables/
    │   └── useScreenSize.ts    # Screen size responsive detection
    ├── stores/
    │   └── gallery.ts          # Pinia store for gallery state
    ├── services/
    │   ├── api.ts              # Axios API client
    │   └── websocket.ts        # WebSocket handler for scan progress
    └── types/
        └── index.ts            # TypeScript interfaces
```

### useScreenSize Composable

Unified screen width detection across components.

**Breakpoints**:
| State | Width Range | Use Case |
|-------|-------------|----------|
| `isMobile` | < 768px | Mobile |
| `isTablet` | 768px - 1024px | Tablet |
| `isDesktop` | >= 1024px | Desktop |

### Gallery Lazy Loading Optimization

Two-level lazy loading for optimal performance:

#### 1. Thumbnail Lazy Loading (MediaCard.vue)

`IntersectionObserver` loads thumbnails only when entering viewport:
- `rootMargin: '200px'` - Preload 200px before entering viewport
- Unobserve after load to free resources

#### 2. Infinite Scroll Trigger (Gallery.vue)

Each gallery column has a sentinel element observed by `IntersectionObserver`:
- `rootMargin: '400px'` - Trigger 400px before reaching bottom
- More efficient than scroll event listeners (no throttling needed)
- Eliminates blank gaps in waterfall layout

### Thumbnail Size System

| Size | Dimension | Use Case |
|------|-----------|----------|
| `small` | Width 300px | Small screens |
| `medium` | Width 600px | Default |
| `large` | Height 900px | High-quality preview |
| `full` | 0 (original) | Full-size transcoded output (no resizing) |

## Testing

### Running Tests

```bash
# Backend tests (Rust)
cd rust
./cargo-with-vendor.sh test --features "vendor-build,video-processing"

# With system libheif (if available)
cargo test

# Frontend - unit tests are limited, primarily manual testing
cd frontend
npm run build  # Type checking
```

### Test Structure

Tests are co-located with source files using Rust's module system:

```
rust/src/
├── services/
│   ├── scan_service.rs     # Tests via #[cfg(test)] module
│   └── file_service.rs     # Tests via #[cfg(test)] module
├── processors/
│   ├── image_processor.rs  # Tests via #[cfg(test)] module
│   └── heif_processor.rs  # Tests via #[cfg(test)] module
└── db/
    └── repository.rs        # Tests via #[cfg(test)] module
```

### Test Fixtures

Test fixtures are defined in `rust/src/fixtures/mod.rs` and provide:
- Sample media files for different formats
- Mock database setup utilities
- Temporary directory management

### Running Specific Tests

```bash
# Run tests matching a pattern
cargo test scan_service

# Run tests with output
cargo test -- --nocapture

# Run doc tests
cargo test --doc
```

## CI/CD

### GitHub Actions Workflow

The project uses GitHub Actions for CI (`.github/workflows/ci.yml`):

**Triggers**:
- Push to `main` branch
- Pull requests to `main` branch

**Jobs**:
1. **Build**: Ubuntu latest
2. **Frontend**: Install dependencies, build (type-checked)
3. **Backend**: Build with vendor libheif and run tests

**CI Command**:
```bash
cd rust
./cargo-with-vendor.sh test --features "vendor-build,video-processing"
```

### Local CI Validation

To validate changes locally before pushing:
```bash
# Full build and test
cd rust && ./cargo-with-vendor.sh test --features "vendor-build,video-processing"
cd frontend && npm install && npm run build
```

## Database

### Scan Flow Overview

| Phase | Value | Description |
|-------|-------|-------------|
| 1. Collecting | `collecting` | Walk directory recursively, collect supported files |
| 2. Counting | `counting` | Batch check DB, compare mtime, count files to add/update/delete |
| 3. Processing | `processing` | Parallel metadata extraction for new/modified files |
| 4. Writing | `writing` | Batch upsert results + batch_touch for unchanged files |
| 5. Deleting | `deleting` | Remove database entries for missing files |
| 6. Completed | `completed` | Scan finished successfully |

### Thread Pool Isolation (Scan Tasks)

All scan tasks run in dedicated thread pools, isolated from web service requests:

| Task Type | Thread Pool | Isolation |
|-----------|-------------|-----------|
| Initial/User/Scheduled scan | `spawn_blocking` + dedicated Runtime | Isolated from API/WebSocket |
| Image transcoding (JPEG/HEIC) | `TranscodingPool` (Rayon, configurable threads) | CPU-intensive tasks |
| API/WebSocket | Tokio async executor | Isolated from scans |

**Note**: Transcoding pool thread count is configurable via `LATTE_TRANSCODING_THREADS` (default: 4).

### Modification Time Comparison

The scanner compares file mtime with database to skip unchanged files:
- `batch_check_exists()` returns: `to_add`, `to_update`, `skip_list`
- `skip_list` files skip expensive metadata extraction
- `batch_touch()` efficiently updates `last_scanned` for skipped files

### Supported File Extensions

- **Images**: `.jpg`, `.jpeg`, `.png`, `.gif`, `.bmp`, `.webp`, `.tiff`, `.heic`, `.heif`
- **Videos**: `.mp4`, `.avi`, `.mov`, `.mkv`, `.wmv`, `.flv`, `.webm`

### Scan Progress Tracking - ScanStateManager

**Global state manager** (`websocket/scan_state.rs`):
- Centralized scan progress tracking
- Business logic sends progress updates via message channel
- Worker broadcasts WebSocket messages to frontend

**Key methods** (`scan_service.rs` calls these):
| Method | Purpose |
|--------|---------|
| `set_phase(phase)` | Update current phase |
| `set_total(total)` | Set total files to process |
| `set_file_counts(add, update, delete)` | Set file counts |
| `reset_counters()` | Reset counters only (no broadcast) |
| `started()` | Mark scan as started |
| `increment_success/failure()` | Increment counters |
| `completed()/error()/cancelled()` | Mark final state |
| `to_progress_message()` | Get current state for HTTP API |

**Message Flow**:
```
scan_service → ProgressUpdate messages → ScanStateManager → WebSocket broadcast
```

**Key Features**:
- Single source of truth for scan state
- Progress broadcast every 10 files
- `files_to_delete` calculated early during Counting phase
- `to_progress_message()` provides HTTP API fallback

### Scan Cancellation Behavior

| Phase | Cancel Action |
|-------|---------------|
| Collecting | Stops collection, returns early |
| Counting | Stops batch check, proceeds with collected data |
| Processing | Saves processed files, then cancels |
| Writing | Finishes current batch, saves partial |
| Deleting | Skips entirely |

### WebSocket Progress Broadcast

**Message Structure**: `ScanProgressMessage` with fields: `scanning`, `phase`, `totalFiles`, `successCount`, `failureCount`, `progressPercentage`, `status`, `filesToAdd`, `filesToUpdate`, `filesToDelete`, `startTime`.

**Broadcast Triggers**:
- Every 10 files during Processing phase
- On phase change
- On state change (`started`, `completed`, `error`, `cancelled`)

### ScanProgressBroadcaster and Shared State

**Circular dependency resolution**:
- `ScanProgressBroadcaster` holds optional `ScanStateManager` reference
- Initialized in `app.rs` using `Arc::make_mut()` to break cycle

**Dual access**:
- WebSocket: Via broadcast channel
- HTTP API: Via `to_progress_message()` using shared state

### WebSocket Connection

- **Endpoint**: `WS /ws/scan`
- **Protocol**: Native WebSocket
- **Client**: `frontend/src/services/websocket.ts` - `ScanProgressWebSocketService`
- **Reconnect**: Automatic every 5 seconds

### Frontend Chinese Progress Text

Phase labels in Chinese:
| Phase | Chinese |
|-------|---------|
| idle | 就绪 |
| collecting | 收集中 |
| counting | 检查中 |
| processing | 处理中 |
| writing | 保存中 |
| deleting | 清理中 |
| completed | 完成 |
| error | 错误 |
| cancelled | 已取消 |

### Scanning Modes

#### Parallel Mode (Default)

| Phase | Operation | Optimization |
|-------|-----------|--------------|
| 1. Collecting | Walk directory | Fast, no DB access |
| 2. Counting | Batch DB check + mtime | Single IN query per 500 files |
| 3. Processing | Parallel extraction | Only modified files, concurrency = CPU cores × 2 |
| 4. Writing | Batch upsert | Transaction per 100 files |
| 5. Deleting | Cleanup | Single DELETE query |

**Performance scenarios**:
| Scenario | Without mtime check | With mtime check |
|----------|---------------------|------------------|
| Full scan (no changes) | O(n) metadata extraction | O(1) batch_touch |
| Few new files | Process all n files | Process only new/modified |

### Configuration

| Variable | Default | Description |
|---------------------|---------|-------------|
| `LATTE_SCAN_WORKER_COUNT` | CPU cores × 2 | Metadata extraction worker count |

## Dependencies

### Rust Backend

| Category | Crate | Version |
|----------|-------|---------|
| Web Framework | axum | 0.8 |
| Async Runtime | tokio | 1 |
| HTTP | tower, tower-http | 0.5, 0.6 |
| Database | sqlx | 0.8 |
| Image Processing | image | 0.25 |
| HEIC/HEIF | libheif-rs | 2.6.1 |
| Video | ffmpeg-next | 7 (optional) |
| Caching | moka | 0.12 |
| Serialization | serde, serde_json | 1 |
| Date/Time | chrono | 0.4 |
| Scheduling | tokio-cron-scheduler | 0.12 |
| EXIF | kamadak-exif (custom fork) | git rev |
| Thread Pool | rayon | 1.11 |
| Logging | tracing, tracing-subscriber | 0.1, 0.3 |
| WebSocket | tokio-tungstenite | 0.28 |
| Utilities | bytes, futures, tokio-util | various |

### Frontend

| Category | Package | Version |
|----------|---------|---------|
| Framework | vue | 3.x |
| State Management | pinia | 2.x |
| UI Components | element-plus | 2.x |
| Build Tool | vite | 5.x |
| HTTP Client | axios | 1.x |
| Icons | @fortawesome/fontawesome-free | 6.x |
| Date Handling | dayjs | 1.x |
| TypeScript | typescript | 5.x |

## Feature Flags

- `video-processing`: Enable FFmpeg video thumbnail support

## Key Design Patterns

### AppState
Shared application state passed via Axum's `State` extractor. Contains all services and config.

### Processor Registry
`ProcessorRegistry` manages media processors via plugin architecture. Processors registered with priority, registry finds matching processor for each file.

### Async Trait Pattern
Media processors use `async-trait` crate to enable async methods in trait objects.

## Project Rules

1. **Code Structure**: Keep simple and clear. Avoid over-abstraction.
2. **Single Responsibility**: Each component has single, clear responsibility.
3. **Code Reuse**: Consolidate duplicate code. DRY principle.
4. **Comments**: Add clear comments for complex key logic.
5. **Testing**: Ensure unit test coverage for functional additions.
6. **Cleanup**: REMOVE obsolete code on time. They would only occur confusion and maintenance cost.
7. **Privacy**: Don't access production databases without explicit authorization.

## Common Tasks

### Add new file format support
1. Create processor implementing `MediaProcessor` trait in `rust/src/processors/`
2. Define supported extensions in `supports()`
3. Set appropriate `priority()` (higher = first)
4. Register in `app.rs` via `ProcessorRegistry`

### Add new EXIF field
1. Add field to `MediaMetadata` in `processor_trait.rs`
2. Extract in processor's `process()` method
3. Add to TypeScript interface in `frontend/src/types/index.ts`
4. Update frontend display in `PhotoViewer.vue`

### Add new API endpoint
1. Define request/response types in appropriate module
2. Add handler function in `rust/src/api/`
3. Register route in `app.rs`
4. Add TypeScript client function in `frontend/src/services/api.ts`
5. Add frontend component if needed

### Modify thumbnail generation
1. Check `rust/src/services/file_service.rs` for thumbnail logic
2. Check `rust/src/utils/thumbnail.rs` for scaling utilities
3. Adjust sizes in environment variables if needed
4. Clear cache if testing: `rm -rf ./cache/`

### Modify scan behavior
1. Check `rust/src/services/scan_service.rs` for scan logic
2. Check `rust/src/websocket/scan_state.rs` for progress tracking
3. Adjust batch sizes in environment variables
4. Test with small photo set first

### DateNavigator Component Logic

- Backend returns dates in **descending order**: newest first
- SQL: `ORDER BY date DESC` in `repository.rs:find_dates_with_files()`

**Navigation**:
| Button | Action | Direction | Condition |
|--------|--------|-----------|-----------|
| Left Arrow | `navigateDate(1)` | Towards older dates | `index < length - 1` |
| Right Arrow | `navigateDate(-1)` | Towards newer dates | `index > 0` |

### PhotoViewer Component - Metadata Display

**Displayed fields**:
| Category | Fields |
|----------|--------|
| Time | `exifTimestamp`, `createTime`, `modifyTime` |
| Device | `cameraMake`, `cameraModel`, `lensModel` |
| Settings | `exposureTime`, `aperture`, `iso`, `focalLength` |
| File | `width`, `height`, `fileSize`, `duration`, `videoCodec` |

**Small screen optimization**: Mobile devices (<768px) only request `large` size thumbnail.

## Guidelines

Before taking ANY motivated change, promise you would understand the following guidelines first.

### Common Development Guideline

 - When modifing any existing code that could infect any existing logic or behavior, do not ignore any potential side effect. MAKE SURE to check all the relevant code, or serious bugs might be introduced.
 - Before any massive code change, evaluate the impact and the RISKS on the existing codebase.
 - After any massive code change, e.g. feature and refactor, or critical logical change & breaking change, MAKE SURE to check and update the logical design to CLAUDE.md for future reference.
 - If you have multiple approach for a hard or systematic problem, programming an rust example to test if it could work in the first place is a good idea.

### Rust Development Guideline

Before using any crates, make sure to accomplish the following steps:

1. Update `Cargo.toml` with the required crate versions. If not sure about the version, do web search, or stop and call the question tool to ask me.
2. Check the crate documentation. First MAKE SURE to run `cargo doc` to build the documentation locally. Then read the documentation under `target/doc/` for the crate you are using. If the document is not accessible, try to read the source code directly.
3. After you are all clear about the crate's functionality, definition and usage associated with your goal, you can proceed with coding.

Remember, just starting developing the code without reading the documentation may lead to unexpected errors or bugs, which is NOT ACCPETABLE. Rust has a decent documentation system, make nice use of it.

After finished the modification, REMEMBER to run `cargo check` and `cargo test` to make sure the program compiles and passes the tests.

### Debug Guideline

 - Debugging must be wide-range, detailed and cautious. Never omit any relevant information and suspicious code when debugging.
 - For those difficult-to-debug issues, giving arbitrary conclusions for the problem is COMPLETELY unacceptable.
   - You should be clear that you are an AI assistant and lack of outer information, might make mistakes and wrong decisions from time to time.
   - Give final conclusions only when you are totally sure you do build up a completed, trusted evidence chain towards the reason. Otherwise always give your conclusion as a GUESS or assumption.
 - Feel free to ask me the user for more information or necessary manual operation if needed.
 - Do not hesitate to add well-detailed debug information, e.g. error messages, stack traces, or any other relevant details when lack of information.
 - When running the program, COLLECT the console output to file for further analysis.

 ### Frontend Design Guideline

  - The design principle of the frontend is to keep it elegant, simple to use, and match with the human intuition.
  - By default the UI should not contain too much bloated information, only show detail when user open the detail view. The detail view should be well organized and easy to read, and the entry is easy to find.
  - Check all the state variable associated with your modification, and make sure they are updated correctly.
  - Since the frontend might be hard to test automatically, ALWAYS give a use case guide to test your modification. e.g. click the button, input the text, or navigate to the page, and check the thing happened as expected.

 ### Git Commit Guideline

  - DO NOT COMMIT ALL the files. Only commit the files modified in the current conversation.
  - When doing major changes, ONLY commit when I comfirm the program works successfully as expected.
  - The commit message should be clear and concise, in several lines of human-style sentences.

## Known Issues

### iPhone mov Video Seek Failure

**Problem**: iPhone mov videos fail to seek to middle/end positions.
**Root cause**: iPhone mov files have moov atom at file end (HEVC/H.265 common practice). Range requests return data without moov metadata.
**Solution**: Use full file request for mov format, or detect moov position server-side.

### Timezone Handling Strategy

**Design principles**:
1. **Frontend display**: Show time literal directly, no timezone conversion
2. **Backend sorting**: Sort by literal time (not UTC)
3. **Timezone hint**: Show timezone label only when different from user's local timezone

**Known issue**: Photos from different timezones may be out of order. Not currently fixed.
