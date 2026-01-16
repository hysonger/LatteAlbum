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
│   ├── broadcast.rs     # ScanProgressBroadcaster for WebSocket broadcasting
│   ├── handler.rs       # WebSocket connection handler
│   ├── progress.rs      # Legacy progress tracker (deprecated, not used)
│   └── scan_state.rs    # ScanStateManager - global scan state management
└── utils/               # Helper functions
```

### Key Services

| Service | Responsibility |
|---------|---------------|
| `FileService` | Thumbnail generation, original file serving |
| `ScanService` | File scanning, metadata extraction |
| `ScanStateManager` | Global scan state management and progress broadcasting |
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
- `GET /api/system/scan/progress` - Scan progress (HTTP fallback, returns `phase` not `phaseMessage`)
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

## File Scanning

### Scan Flow Overview

The scanning process consists of 6 phases:

| Phase | Phase Value | Description |
|-------|-------------|-------------|
| 1. Collecting | `collecting` | Walk directory recursively, collect supported files |
| 2. Counting | `counting` | Batch check DB, compare mtime, count files to add/update/delete |
| 3. Processing | `processing` | Parallel metadata extraction for new/modified files |
| 4. Writing | `writing` | Batch upsert results + batch_touch for unchanged files |
| 5. Deleting | `deleting` | Remove database entries for missing files |
| 6. Completed | `completed` | Scan finished successfully |

### Modification Time Comparison

The scanner optimizes performance by comparing file modification times:

1. **batch_check_exists()** returns three categories:
   - `to_add`: Files not in database
   - `to_update`: Files with changed `mtime`
   - `skip_list`: Files with unchanged `mtime` (only update `last_scanned`)

2. **Only modified files** trigger metadata extraction:
   - Files in `skip_list` skip expensive metadata extraction
   - `batch_touch()` efficiently updates `last_scanned` in batches

3. **Time comparison logic**:
   ```rust
   // Compare filesystem mtime with database modify_time
   let fs_time = fs_modify_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
   let db_time = existing.modify_time.map(|t| t.timestamp() as u64).unwrap_or(0);

   if fs_time == db_time {
       // Unchanged - skip metadata extraction
       skip_list.push(path.clone());
   } else {
       // Changed - needs metadata extraction
       to_update += 1;
   }
   ```

### Supported File Extensions

- **Images**: `.jpg`, `.jpeg`, `.png`, `.gif`, `.bmp`, `.webp`, `.tiff`, `.heic`, `.heif`
- **Videos**: `.mp4`, `.avi`, `.mov`, `.mkv`, `.wmv`, `.flv`, `.webm`

### Scan Progress Tracking - ScanStateManager

**Global Scan State Manager** (`websocket/scan_state.rs`):

The `ScanStateManager` is a global state manager that centralizes scan progress tracking. Business logic sends progress updates to it via a message channel, and it broadcasts WebSocket messages to frontend.

```rust
/// Scan phases
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanPhase {
    Idle,
    Collecting,
    Counting,
    Processing,
    Writing,
    Deleting,
    Completed,
    Error,
    Cancelled,
}

/// Scan state (shared via Arc<RwLock>)
pub struct ScanState {
    pub phase: ScanPhase,
    pub phase_message: String,
    pub scanning: bool,
    pub total_files: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub files_to_add: u64,
    pub files_to_update: u64,
    pub files_to_delete: u64,
    pub start_time: Option<String>,
}

/// Progress update messages (business logic -> state manager)
pub enum ProgressUpdate {
    SetPhase(ScanPhase, String),
    SetTotal(u64),
    IncrementSuccess,
    IncrementFailure,
    SetFileCounts(u64, u64, u64), // add, update, delete
    Started,
    Completed,
    Error(String),
    Cancelled,
}

/// Scan state manager
pub struct ScanStateManager {
    state: Arc<RwLock<ScanState>>,
    progress_sender: mpsc::Sender<ProgressUpdate>,
    _worker_task: AbortHandle,
}
```

**Key methods** (`scan_service.rs` calls these methods):
| Method | Purpose |
|--------|---------|
| `set_phase(phase, message)` | Update current phase (message stored but not sent to frontend) |
| `set_total(total)` | Set total files to process |
| `set_file_counts(add, update, delete)` | Set file counts for add/update/delete |
| `started()` | Mark scan as started, set start_time |
| `increment_success()` | Increment success count |
| `increment_failure()` | Increment failure count |
| `completed()` | Mark scan as completed |
| `error(message)` | Mark scan as error (message not sent to frontend) |
| `cancelled()` | Mark scan as cancelled |
| `get_state()` | Get current state (for HTTP fallback) |

**Note**: The `message` parameter in `set_phase()` and `error()` is stored in `ScanState.phase_message` but is NOT sent to the frontend. The frontend generates Chinese display text locally.

**Message Flow**:
```
scan_service (business logic)
    │
    ├── set_phase(Collecting, "Collecting files...")
    ├── set_total(318)
    ├── set_phase(Counting, "Checking database...")
    ├── set_file_counts(302, 0, 16)  // Early calculation of delete count
    ├── set_phase(Processing, "...")
    ├── started()
    ├── increment_success/failure()  // Every 10 files
    ├── set_phase(Writing, "Saving to database...")
    ├── set_phase(Deleting, "Cleaning up...")
    └── completed()
            │
            ▼
    ScanStateManager (worker task)
            │
            ├── Updates shared state (Arc<RwLock<ScanState>>)
            ├── Broadcasts every 10 files or on phase change
            └── Sends to WebSocket clients via ScanProgressBroadcaster
```

**Key Features**:
- Single source of truth for scan state
- Ordered progress updates via mpsc channel
- Progress broadcast every 10 files (configurable in `scan_state.rs`)
- `files_to_delete` calculated early during Counting phase via `count_missing()`
- `files_to_add/update/delete` set once and preserved throughout scan

### WebSocket Progress Broadcast

**Message Structure** (`websocket/broadcast.rs`):
```rust
#[serde(rename_all = "camelCase")]
pub struct ScanProgressMessage {
    pub scanning: bool,
    pub phase: Option<String>,              // collecting, counting, processing, writing, deleting, completed
    // phase_message 已移除，由前端根据 phase 值生成中文文本
    pub total_files: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub progress_percentage: String,        // e.g., "45.50"
    pub status: String,                     // started, progress, completed, error, cancelled
    pub files_to_add: u64,
    pub files_to_update: u64,
    pub files_to_delete: u64,
    pub start_time: Option<String>,         // ISO 8601 timestamp
}
```

**JSON Field Names**: All fields use camelCase (e.g., `totalFiles`, `successCount`, `progressPercentage`) due to `#[serde(rename_all = "camelCase")]`.

**Broadcast Methods** (`websocket/scan_state.rs` handles all broadcasts):

Progress broadcasts are triggered automatically by `ScanStateManager` worker task:
- **Every 10 files**: During Processing phase
- **On phase change**: When `set_phase()` is called
- **On state change**: When `started()`, `completed()`, `error()`, `cancelled()` are called

The worker task constructs `ScanProgressMessage` from current state and sends via `ScanProgressBroadcaster`.

### WebSocket Connection

- **Endpoint**: `WS /ws/scan`
- **Protocol**: Native WebSocket (not STOMP/SockJS like Java version)
- **Client**: `frontend/src/services/websocket.ts` - `ScanProgressWebSocketService` class
- **Reconnect**: Automatic reconnection every 5 seconds if connection lost

### Frontend Chinese Progress Text

**The backend no longer sends `phase_message`** - The frontend generates Chinese display text locally based on the `phase` value.

**Phase Label Mapping** (`frontend/src/services/websocket.ts`):
```typescript
export const PHASE_LABELS: Record<string, string> = {
  idle: '空闲',
  collecting: '收集中',
  counting: '检查中',
  processing: '处理中',
  writing: '保存中',
  deleting: '清理中',
  completed: '完成',
  error: '错误',
  cancelled: '已取消',
}
```

**Chinese Message Generator** (`frontend/src/services/websocket.ts`):
```typescript
export function getPhaseMessage(progress: Partial<ScanProgressMessage>): string {
  const phase = progress.phase?.toLowerCase()

  if (!phase || phase === 'idle') return '就绪'
  if (progress.status === 'error') return '扫描出错'
  if (progress.status === 'cancelled') return '扫描已取消'
  if (progress.status === 'completed') return '扫描完成'

  // Processing: show progress info
  const processed = (progress.successCount || 0) + (progress.failureCount || 0)
  const total = progress.totalFiles || 0
  if (total > 0) {
    return `${PHASE_LABELS[phase] || phase} (${processed}/${total}, ${progress.progressPercentage}%)`
  }
  return PHASE_LABELS[phase] || phase
}
```

**Display Example**:
| Phase | Chinese Display |
|-------|-----------------|
| idle | 就绪 |
| collecting | 收集中 (120/318, 37.73%) |
| counting | 检查中 |
| processing | 处理中 (50/302, 16.56%) |
| writing | 保存中 |
| deleting | 清理中 |
| completed | 扫描完成 |
| error | 扫描出错 |
| cancelled | 扫描已取消 |

### Progress Update Frequency

**Progress updates are sent by `ScanStateManager` worker task**:
- **Every 10 files**: During Processing phase, counting success + failures
- **On phase change**: Any call to `set_phase()` triggers immediate broadcast
- **On state change**: `started()`, `completed()`, `error()`, `cancelled()` trigger immediate broadcast
- **Final broadcast**: On completion with preserved phase information

**Percentage calculation**: `(success_count + failure_count) / total_files * 100`

### Scanning Modes

#### Parallel Mode (Default)

The parallel scanning implementation with mtime optimization significantly improves performance:

| Phase | Operation | Optimization |
|-------|-----------|--------------|
| 1. Collecting | Walk directory recursively | Fast, no DB access |
| 2. Counting | Batch DB check + mtime comparison | Single IN query per 500 files |
| 3. Processing | Parallel metadata extraction | Only modified files, concurrency = CPU cores × 2 |
| 4. Writing | Batch upsert + batch_touch | Transaction per 100 files, skip_list updated in batches |
| 5. Deleting | Cleanup missing files | Single DELETE query |

**Key optimizations:**
- Batch database queries instead of O(n) individual queries
- Modification time comparison skips unchanged files
- `batch_touch()` updates `last_scanned` for unchanged files without re-extraction
- Parallel metadata extraction with semaphore-controlled concurrency
- Batch writes reduce transaction overhead

**Performance improvement scenarios:**

| Scenario | Without mtime check | With mtime check |
|----------|---------------------|------------------|
| Full scan (no changes) | O(n) metadata extraction | O(1) batch_touch |
| Few new files | Process all n files | Process only new/modified |
| Modified files | Re-extract all | Re-extract only changed |

#### Serial Mode (Fallback)

Enabled when `LATTE_SCAN_PARALLEL=false`. Processes files one-by-one with individual database operations. Used as fallback for debugging or problematic environments.

### Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `LATTE_SCAN_PARALLEL` | `true` | Enable parallel scanning |
| `LATTE_SCAN_CONCURRENCY` | CPU cores × 2 | Metadata extraction concurrency |

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
    pub scan_state: Arc<ScanStateManager>,  // Global scan state manager
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

### DateNavigator Component Logic

The `DateNavigator.vue` component provides date navigation with left/right arrow buttons. Key implementation details:

**Data Structure**:
- Backend returns dates in **descending order**: `dates[0]` = newest date, `dates[length-1]` = oldest date
- SQL: `ORDER BY date DESC` in `repository.rs:find_dates_with_files()`

**Navigation Logic**:
| Button | Action | Direction | Condition |
|--------|--------|-----------|-----------|
| Left Arrow | `navigateDate(1)` | Index +1 (towards older dates) | `canNavigatePrev`: `index < length - 1` |
| Right Arrow | `navigateDate(-1)` | Index -1 (towards newer dates) | `canNavigateNext`: `index > 0` |

**Button States**:
| Current Position | Left Button | Right Button |
|------------------|-------------|--------------|
| Newest (index=0) | Enabled | Disabled |
| Oldest (index=length-1) | Disabled | Enabled |
| Middle | Enabled | Enabled |

## Common Development Guideline

 - After any massive code change, e.g. feature and refactor, or critical logical change & breaking change, MAKE SURE to check and update the logical design to CLAUDE.md for future reference.
 - If you have multiple approach for a hard or systematic problem, programming an rust example to test if it could work in the first place is a good idea.

## Rust Development Guideline

Before using any crates, make sure to accomplish the following steps:

1. Update `Cargo.toml` with the required crate versions. If not sure about the version, do web search, or stop and call the question tool to ask me.
2. Check the crate documentation. First MAKE SURE to run `cargo doc` to build the documentation locally. Then read the documentation under `target/doc/` for the crate you are using. If the document is not accessible, try to read the source code directly.
3. After you are all clear about the crate's functionality, definition and usage associated with your goal, you can proceed with coding. 

Remember, just starting developing the code without reading the documentation may lead to unexpected errors or bugs, which is NOT ACCPETABLE. Rust has a decent documentation system, make nice use of it.

## Debug Guideline

 - For those difficult-to-debug issues, DO NOT give arbitrary conclusions for the problem. 
   - You should be clear that you are an AI assistant and lack of outer information, might make mistakes from time to time. 
   - Tend to give assumptions and suggestions rather than conclusions, unless you indeed find the actual valid information of the reason. Feel free to ask me the user for more information if needed.
 - Do not hesitate to add well-detailed debug information, e.g. error messages, stack traces, or any other relevant details when lack of information. 
   - After I claim the problem is resolved, revert the temporary debug information.
