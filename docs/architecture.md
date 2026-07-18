# Architecture Guide

Project architecture, design patterns, and core system design for Latte Album.

## Project Overview

Latte Album is a personal photo album application for NAS deployment. Backend: Rust (Axum, SQLx, SQLite). Frontend: Vue 3 + TypeScript + Element Plus.

**Version**: 0.1.0 | **License**: MIT

### Key Features

- Responsive masonry gallery with dual-level lazy loading
- Support for images (JPEG, PNG, GIF, WebP, TIFF, HEIC/HEIF) and videos
- High-performance parallel file scanning with mtime comparison
- Real-time scan progress via WebSocket
- EXIF metadata extraction
- Three-tier thumbnail caching (memory, disk, dynamic generation)
- Scheduled automatic scanning (daily at 2 AM)

### Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Axum 0.8, Rust, Tokio, SQLx, SQLite |
| Frontend | Vue 3, TypeScript, Pinia, Element Plus |
| Image Processing | image crate, libheif-rs |
| Video Processing | FFmpeg (ffmpeg-next) |
| Cache | Moka (memory) + disk cache |

## Backend Structure

```
rust/src/
├── main.rs              # Entry point
├── app.rs               # App struct and router configuration
├── config.rs            # Configuration loading
├── api/                 # REST API handlers (files, directories, system)
├── db/                  # Database layer (pool, models, repository, migrations)
├── services/            # Business logic (scan, file, cache, scheduler, transcoding_pool)
├── processors/          # Media format handlers (plugin architecture)
├── websocket/           # WebSocket for scan progress
├── extraction/          # Metadata utilities
└── fixtures/            # Test fixtures
```

## Key Services

| Service | Responsibility |
|---------|---------------|
| `FileService` | Thumbnail generation, original file serving |
| `ScanService` | File scanning, metadata extraction |
| `ScanStateManager` | Global scan state management and progress broadcasting |
| `CacheService` | Moka-based thumbnail caching |
| `Scheduler` | Daily 2 AM scheduled scan |
| `TranscodingPool` | Rayon-based thread pool for CPU-intensive image processing |

## Frontend Structure

```
frontend/src/
├── views/HomeView.vue         # Main page with Gallery + PhotoViewer
├── components/                 # Gallery, PhotoViewer, MediaCard, DateNavigator, etc.
├── composables/useScreenSize.ts # Screen size detection (mobile/tablet/desktop)
├── composables/useImageZoom.ts  # Image zoom/pan (wheel w/ trackpad-adaptive speed, Ctrl±, drag, touch pinch); images only
├── stores/gallery.ts          # Pinia store for gallery state
└── services/                  # API client and WebSocket handler
```

## Core Design

### Scan Flow

| Phase | Description |
|-------|-------------|
| 1. Collecting | Walk directory recursively, collect supported files |
| 2. Counting | Batch check DB, compare mtime, count files to add/update/delete |
| 3. Processing | Parallel metadata extraction for new/modified files |
| 4. Writing | Batch upsert results + batch_touch for unchanged files |
| 5. Deleting | Remove database entries for missing files |

**Optimization**: Scanner compares file mtime with database to skip unchanged files. Only new/modified files trigger expensive metadata extraction.

### Media Processor Plugin Architecture

Processors implement `MediaProcessor` trait and are registered in `app.rs` via `ProcessorRegistry`. Higher priority matches first.

| Processor | Extensions | Priority |
|-----------|------------|----------|
| `HeifImageProcessor` | .heic, .heif | 100 |
| `StandardImageProcessor` | .jpg, .jpeg, .png, .gif, .bmp, .webp, .tiff | 10 |
| `VideoProcessor` | .mp4, .avi, .mov, .mkv, .wmv, .flv, .webm | 10 |

### Thread Pool Isolation

| Task Type | Thread Pool |
|-----------|-------------|
| Initial/User/Scheduled scan | `spawn_blocking` + dedicated Runtime |
| Image transcoding (JPEG/HEIC) | `TranscodingPool` (Rayon, configurable threads) |
| API/WebSocket | Tokio async executor |

### Thumbnail Caching Strategy

| Tier | Storage | Response |
|------|---------|----------|
| L1 | Memory cache (Moka) | Direct return |
| L2 | Disk cache | `File::open` + `ReaderStream` (32KB chunks) |
| L3 | Dynamic generation | Write to cache, then return |

### File Streaming

- **Original files**: HTTP Range requests (206 Partial Content), large files (>50MB) use `ReaderStream`
- **Thumbnails**: Three-tier caching (see above)

### Scan Progress Tracking

`ScanStateManager` (`websocket/scan_state.rs`) provides:
- Centralized scan progress tracking
- WebSocket broadcast every 10 files during processing
- HTTP API fallback via `to_progress_message()`

### Gallery Lazy Loading

Two-level lazy loading:
1. **Thumbnail lazy loading** (MediaCard.vue): `IntersectionObserver` with 200px preload
2. **Infinite scroll** (Gallery.vue): Sentinel elements with 400px trigger

## API Endpoints

### File Operations

- `GET /api/files` - List with pagination, sorting, filtering
- `GET /api/files/dates` - Get dates with photos
- `GET /api/files/{id}` - File details
- `GET /api/files/{id}/thumbnail?size={small|medium|large|full}` - Thumbnail stream
- `GET /api/files/{id}/original` - Original file stream with Range support
- `GET /api/files/{id}/neighbors` - Prev/next for navigation
- `GET /api/directories` - Directory tree

### System Operations

- `POST /api/system/rescan` - Trigger directory rescan
- `POST /api/system/scan/cancel` - Cancel ongoing scan
- `GET /api/system/status` - System status
- `GET /api/system/scan/progress` - Scan progress (HTTP fallback)
- `WS /ws/scan` - WebSocket for real-time scan progress

## Dependencies

### Key Rust Crates

axum 0.8, tokio 1, sqlx 0.8, image 0.25, libheif-rs 2.6.1, moka 0.12, serde 1, chrono 0.4, kamadak-exif (custom fork), rayon 1.11

### Key Frontend Packages

vue 3.x, pinia 2.x, element-plus 2.x, vite 5.x, axios 1.x

## Common Tasks

### Add new file format support

1. Create processor implementing `MediaProcessor` trait in `rust/src/processors/`
2. Define supported extensions in `supports()`
3. Set appropriate `priority()` (higher = first)
4. Register in `app.rs` via `ProcessorRegistry`

### Add new API endpoint

1. Define handler in `rust/src/api/`
2. Register route in `app.rs`
3. Add TypeScript client function in `frontend/src/services/api.ts`

### Modify scan behavior

1. Check `rust/src/services/scan_service.rs` for scan logic
2. Check `rust/src/websocket/scan_state.rs` for progress tracking
3. Test with small photo set first
