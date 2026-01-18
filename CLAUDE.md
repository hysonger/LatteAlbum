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
| `LATTE_THUMBNAIL_LARGE` | `900` | Large thumbnail height (px) - fixed height, maintains aspect ratio |
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

### Image Transcoding and EXIF Extraction

#### Thumbnail Generation Pipeline

| Format | Decoder | Scaling | Encoder | Output |
|--------|---------|---------|---------|--------|
| JPEG/PNG/GIF/WebP/TIFF | `image` crate | `thumbnail()` (fast integer) | JPEG (80% quality) | JPEG |
| HEIC/HEIF | `libheif-rs` | `image.scale()` (libheif built-in) | JPEG (80% quality) | JPEG |

#### 缩略图生成技术细节

**缩放模式**：
- `small`/`medium`：按固定宽度缩放，高度按比例计算
- `large`：按固定高度缩放，宽度按比例计算（适合大图预览占位）

**标准图片 (JPEG/PNG/GIF/WebP/TIFF)** (`image_processor.rs`):
```rust
// 使用 image crate 解码
let img = ImageReader::open(path)?.decode()?;

// thumbnail(w, h) 缩放到不超过 w×h 范围，保持宽高比
let thumb = if fit_to_height {
    // fit_to_height=true: 按固定高度缩放
    let ratio = img.width() as f64 / img.height() as f64;
    let target_width = (target_size as f64 * ratio) as u32;
    img.thumbnail(target_width, target_size)
} else {
    // fit_to_height=false: 按固定宽度缩放
    img.thumbnail(target_size, u32::MAX)
};

// 编码为 JPEG
let encoder = JpegEncoder::new_with_quality(&mut bytes, (quality * 100.0) as u8);
```

**HEIC/HEIF 图片** (`heif_processor.rs`):
```rust
// 使用 libheif-rs 解码
let ctx = HeifContext::read_from_file(&path_str)?;
let handle = ctx.primary_image_handle()?;
let image = lib_heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None)?;

// 根据模式计算目标尺寸
let (target_w, target_h) = if fit_to_height {
    // fit_to_height=true: 按固定高度缩放
    let ratio = image.width() as f64 / image.height() as f64;
    ((target_size as f64 * ratio) as u32, target_size)
} else {
    // fit_to_height=false: 按固定宽度缩放
    (target_size, (target_size as f64 * (image.height() as f64 / image.width() as f64)) as u32)
};

// 使用 libheif 内置缩放
let scaled = image.scale(target_w, target_h, None)?;

// 转换为 RGBA → RGB → JPEG
let rgb_image = DynamicImage::ImageRgba8(rgba_image).to_rgb8();
```

**性能特点**:
- HEIC 解码速度约为 JPG 的 170x (3ms vs 555ms)
- JPEG 编码速度约为 WebP 的 3-7x
- 缩略图使用 `spawn_blocking` 在阻塞线程池中执行，避免阻塞异步 Runtime

#### EXIF 元数据提取

**使用库**: `kamadak-exif` (crates.io) / `exif` (GitHub package name)

**提取字段**:
| 字段 | EXIF Tag | 说明 |
|------|----------|------|
| `exif_timestamp` | DateTimeOriginal | 拍摄时间 |
| `exif_timezone_offset` | OffsetTimeOriginal | 拍摄时区 |
| `camera_make` | Make | 相机厂商 |
| `camera_model` | Model | 相机型号 |
| `lens_model` | LensModel | 镜头型号 |
| `aperture` | FNumber | 光圈值 |
| `exposure_time` | ExposureTime | 快门速度 |
| `iso` | ISOSpeedRatings | ISO 感光度 |
| `focal_length` | FocalLength | 焦距 |

**实现位置**:
- 通用提取函数: `image_processor.rs:extract_exif()`
- HEIC 格式: 复用相同的 `extract_exif()` 函数
- 支持格式: JPEG, HEIC, TIFF, PNG 等

**已知限制**: 部分 HEIC 文件的 EXIF 数据块体积非常大，可达10KB以上，导致解析失败。kamadak-exif 已经在最新 commit 中修复了此问题，但目前还没有发布新版本。目前采用直接拉取最新代码编译的方式

#### 文件流式传输

**视频流式播放** (`api/files.rs:get_original`):
- 支持 HTTP Range 请求（206 Partial Content），实现边下载边播放
- 支持 seek 拖动进度条（浏览器自动发送新的 Range 请求）
- 大文件（>50MB）使用 `ReaderStream` 流式传输，避免一次性加载到内存
- 返回 `Accept-Ranges: bytes` 头部告知浏览器支持范围请求

**缩略图流式传输** (`api/files.rs:get_thumbnail`):
采用三级缓存策略：
| 层级 | 存储位置 | 响应方式 |
|------|---------|---------|
| L1 | 内存缓存 (Moka) | 直接返回数据（已在内存中） |
| L2 | 磁盘缓存 | `File::open` + `ReaderStream` 流式传输 |
| L3 | 动态生成 | 写入缓存后返回 |

```rust
// 磁盘缓存流式传输示例
let file = File::open(&disk_path).await?;
let stream = ReaderStream::with_capacity(file, 32 * 1024); // 32KB chunks
Body::from_stream(stream)
```

**缓存服务方法** (`services/cache_service.rs`):
- `get_thumbnail()` - 获取缩略图（自动检查内存+磁盘缓存）
- `get_thumbnail_disk_path()` - 获取磁盘缓存路径（用于流式传输）
- `has_thumbnail()` - 检查缓存是否存在

### 缓存与缩略图性能优化

本节描述缩略图生成和缓存机制的数据流优化策略。

#### 优化概览

| 优化项 | 文件 | 效果 |
|--------|------|------|
| HEIC stride 处理 | `heif_processor.rs` | stride 匹配时避免数据复制 |
| CacheService Bytes | `cache_service.rs` | 内存缓存使用 Bytes 实现 O(1) 克隆 |

#### 1. HEIC stride 优化

libheif 解码 RGBA 数据时可能包含 stride padding（行对齐填充）。如果不处理，会导致图片在斜向方向被严重拉伸。

当 stride == width * 4 时，数据是紧密排列的，无需进行处理，可以直接复制。

```rust
// heif_processor.rs
let bytes_per_row = width as usize * 4;

// 只克隆一次，后续使用所有权数据
let owned_data: Vec<u8> = interleaved.data.to_vec();

let rgba_image = if stride == bytes_per_row {
    // 紧密排列，直接使用所有权数据
    image::RgbaImage::from_raw(width, height, owned_data)
} else {
    // 有 padding，需要逐行复制去除填充
    let mut rgb_data = Vec::with_capacity(width as usize * height as usize * 4);
    for row in 0..height as usize {
        let row_offset = row * stride;
        rgb_data.extend_from_slice(&owned_data[row_offset..row_offset + bytes_per_row]);
    }
    image::RgbaImage::from_raw(width, height, rgb_data)
};
```

**优化效果**: 避免在 stride 匹配时进行不必要的数据复制，减少 HEIC 处理时的内存峰值。

#### 2. CacheService Bytes 优化

使用 `bytes::Bytes` 替代 `Vec<u8>` 作为内存缓存类型。Bytes 基于引用计数，克隆操作是 O(1) 而非 O(n)。

```rust
// cache_service.rs
use bytes::Bytes;

pub struct CacheService {
    memory_cache: Arc<Cache<String, Bytes>>,  // 使用 Bytes
    disk_cache_dir: PathBuf,
}

pub async fn get_thumbnail(&self, file_id: &str, size: &str) -> Option<Bytes> {
    // L1: 检查内存缓存
    if let Some(data) = self.memory_cache.get(&cache_key).await {
        return Some(data);  // Bytes 克隆 O(1)
    }

    // L2: 检查磁盘缓存
    if let Ok(data) = fs::read(&disk_path).await {
        let bytes = Bytes::from(data);  // 从 Vec 转换为 Bytes
        self.memory_cache.insert(cache_key.clone(), bytes.clone()).await;
        return Some(bytes);
    }
    None
}

pub async fn put_thumbnail_bytes(&self, file_id: &str, size: &str, data: Bytes) -> std::io::Result<()> {
    self.memory_cache.insert(cache_key.clone(), data.clone()).await;
    let disk_path = self.disk_cache_dir.join(&cache_key);
    fs::write(&disk_path, &data).await?;
    Ok(())
}
```

**优化效果**:
- 内存缓存命中时的数据返回无需复制
- 磁盘缓存命中时插入内存缓存是 O(1) 克隆
- API 层 `file_service.rs` 在返回前调用 `.to_vec()` 转换为 Vec<u8>

#### 3. 数据流对比

| 场景 | 优化前 | 优化后 |
|------|--------|--------|
| 缓存命中（内存） | 返回 `Vec<u8>` | 返回 `Bytes`（直接返回，无复制） |
| 缓存命中（磁盘） | `read → clone → insert` | `read → Bytes::from → clone` |
| HEIC 处理 | 总是 `interleaved.data.clone()` | stride 匹配时直接使用所有权数据 |
| 内存缓存克隆 | `Vec<u8>` 克隆 O(n) | `Bytes` 克隆 O(1) |

### API Endpoints

**File Operations**:
- `GET /api/files` - List with pagination, sorting, filtering
- `GET /api/files/dates` - Get dates with photos
- `GET /api/files/{id}` - File details
- `GET /api/files/{id}/thumbnail?size={small|medium|large|full}` - Thumbnail stream (supports streaming from disk cache)
- `GET /api/files/{id}/original` - Original file stream with Range support (for video streaming)
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
│   ├── Gallery.vue      # Masonry layout gallery with lazy loading
│   ├── PhotoViewer.vue  # Fullscreen viewer
│   ├── MediaCard.vue    # Thumbnail card with Intersection Observer
│   └── DateNavigator.vue # Calendar date picker
├── stores/
│   └── gallery.ts       # Pinia store for gallery state
├── services/
│   ├── api.ts           # Axios API client
│   └── websocket.ts     # WebSocket handler
└── types/
    └── index.ts         # TypeScript interfaces
```

### Gallery Lazy Loading Optimization

The gallery implements two-level lazy loading for optimal performance:

#### 1. Thumbnail Lazy Loading (MediaCard.vue)

Each `MediaCard` uses `IntersectionObserver` to load thumbnails only when they enter the viewport:

```typescript
// MediaCard.vue
onMounted(() => {
  observer.value = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          loadThumbnail()
          observer.value?.unobserve(entry.target)  // Stop observing after load
        }
      })
    },
    { rootMargin: '200px', threshold: 0 }  // Preload 200px before entering viewport
  )

  if (cardRef.value) {
    observer.value.observe(cardRef.value)
  }
})
```

**Benefits**:
- Reduces initial bandwidth usage
- Only loads images visible or near-visible in viewport
- Improves scrolling performance on large albums

#### 2. Infinite Scroll Trigger (Gallery.vue)

Instead of scroll event listeners, each gallery column has a sentinel element observed by `IntersectionObserver`:

```typescript
// Gallery.vue - Template
<div
  v-for="column in columns"
  :key="column.id"
  class="gallery-column"
>
  <MediaCard ... />
  <!-- Sentinel at bottom of each column -->
  <div
    v-if="displayHasMore"
    :ref="(el) => setColumnSentinel(el, column.id)"
    class="column-sentinel"
  ></div>
</div>

// Script
sentinelObserver.value = new IntersectionObserver(
  (entries) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        loadMore()
      }
    })
  },
  { rootMargin: '400px', threshold: 0 }  // Trigger 400px before reaching bottom
)
```

**Benefits**:
- More efficient than scroll event listeners (no scroll throttling needed)
- Triggers loading when any column approaches viewport bottom
- Eliminates blank gaps in waterfall layout
- Layout changes trigger re-observation of sentinels

**Comparison**:
| Metric | Before | After |
|--------|--------|-------|
| Initial requests | All visible + off-screen | Only viewport + 200px |
| Scroll trigger | Scroll event + debounce | IntersectionObserver |
| Trigger point | Page bottom - 200px | Any column bottom + 400px |

### Thumbnail Size System

| Size | Dimension | Use Case |
|------|-----------|----------|
| `small` | Width 300px | thumbnails for small screens |
| `medium` | Width 600px | Default |
| `large` | Height 900px | High-quality preview (fixed height, maintains aspect ratio) |
| `full` | 0 (original) | Full-size transcoded for photo viewer |

**缩放行为**：
- `small`/`medium`：按固定宽度缩放，高度按比例计算
- `large`：按固定高度缩放，宽度按比例计算

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

### Thread Pool Isolation (Scan Tasks)

All scan tasks run in a dedicated thread pool, isolated from web service requests:

| Task Type | Thread Pool | Isolation |
|-----------|-------------|-----------|
| Initial scan (first run) | `spawn_blocking` + dedicated Runtime | Isolated from API/WebSocket |
| User-triggered scan | `spawn_blocking` + dedicated Runtime | Isolated from API/WebSocket |
| Scheduled scan | `spawn_blocking` + dedicated Runtime | Isolated from API/WebSocket |
| API requests | Tokio async executor | Isolated from scans |
| WebSocket messages | Tokio async executor | Isolated from scans |

**Implementation** (`app.rs` / `system.rs`):
```rust
// Scan tasks use spawn_blocking with a new Runtime
tokio::task::spawn_blocking(move || {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        scan_service.scan(parallel).await;
    });
});
```

This prevents CPU-intensive or I/O-intensive scan operations from blocking API requests.

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
#[derive(Debug, Clone, Default)]
pub struct ScanState {
    pub phase: ScanPhase,
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
#[derive(Debug)]
pub enum ProgressUpdate {
    SetPhase(ScanPhase),
    SetTotal(u64),
    IncrementSuccess,
    IncrementFailure,
    SetFileCounts(u64, u64, u64), // add, update, delete
    ResetCounters,  // 仅重置计数器，不发送广播
    Started,
    Completed,
    Error,
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
| `set_phase(phase)` | Update current phase |
| `set_total(total)` | Set total files to process |
| `set_file_counts(add, update, delete)` | Set file counts for add/update/delete |
| `reset_counters()` | Reset success/failure counters only (no broadcast) |
| `started()` | Mark scan as started, set start_time, reset counters |
| `increment_success()` | Increment success count |
| `increment_failure()` | Increment failure count |
| `completed()` | Mark scan as completed |
| `error()` | Mark scan as error |
| `cancelled()` | Mark scan as cancelled |
| `to_progress_message()` | Convert current state to ScanProgressMessage (for HTTP API) |

**Message Flow**:
```
scan_service (business logic)
    │
    ├── set_phase(Collecting)
    ├── set_total(318)
    ├── set_phase(Counting)
    ├── set_file_counts(302, 0, 16)  // Early calculation of delete count
    ├── set_phase(Processing)
    ├── started()
    ├── increment_success/failure()  // Every 10 files
    ├── set_phase(Writing)
    ├── set_phase(Deleting)
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
- `to_progress_message()` provides current state via HTTP API without broadcast channel

### Scan Cancellation Behavior

**Cancel Flow** (`scan_service.rs`):

When a scan is cancelled, the following behavior occurs:

| Phase | Cancel Action |
|-------|---------------|
| Collecting | Stops file collection, returns early |
| Counting | Stops batch check, proceeds with collected data |
| Processing | Saves processed files, then cancels |
| Writing | Finishes current batch, saves partial results, skips remaining |
| Deleting | Skips delete phase entirely |

**Backend Cancel Logic**:

1. **Serial Mode** (`process_serial`):
   - Detects cancel flag in loop
   - Calls `save_partial_results()` to persist processed files
   - Updates `last_scanned` for unprocessed files via `batch_touch()`
   - Calls `scan_state.cancelled()`

2. **Parallel Mode** (`perform_scan_parallel`):
   - `batch_write_results_with_skip()` detects cancel during batch processing
   - Returns `true` to indicate cancellation
   - Caller checks return value and `is_cancelled` flag
   - Calls `scan_state.cancelled()` instead of `completed()`

**Frontend Cancel Handling**:

| State | Progress Ring | Refresh Icon | Popup |
|-------|---------------|--------------|-------|
| `progress` | Visible | Spinning | Visible on toggle |
| `completed` | Hidden | Checkmark | Hidden |
| `error` | Hidden | X mark | Hidden |
| `cancelled` | Hidden | Default (sync icon) | Hidden immediately |

```typescript
// Frontend status handling
case 'cancelled':
  showScanPopup.value = false  // Immediately hide popup
  refreshStatus.value = 'default'
  scanProgressData.value.status = 'idle'
  break
```

### WebSocket Progress Broadcast

**Message Structure** (`websocket/broadcast.rs`):
```rust
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressMessage {
    pub scanning: bool,
    pub phase: Option<String>,              // collecting, counting, processing, writing, deleting, completed
    pub total_files: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub progress_percentage: String,        // e.g., "45.50"
    pub status: String,                     // idle, progress, completed, error, cancelled
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
- **ResetCounters does NOT broadcast**: Only resets internal counters

The worker task constructs `ScanProgressMessage` from current state and sends via `ScanProgressBroadcaster`.

### ScanProgressBroadcaster and Shared State

**Circular Dependency Resolution** (`websocket/broadcast.rs`):

`ScanProgressBroadcaster` holds an optional reference to `ScanStateManager`:

```rust
#[derive(Clone)]
pub struct ScanProgressBroadcaster {
    tx: broadcast::Sender<ScanProgressMessage>,
    scan_state: Option<Arc<ScanStateManager>>,  // Shared state reference
}

impl ScanProgressBroadcaster {
    /// Set the scan_state reference (must be called after creating ScanStateManager)
    pub fn set_scan_state(&mut self, scan_state: Arc<ScanStateManager>) {
        self.scan_state = Some(scan_state);
    }

    /// Get current progress state (uses shared state, not broadcast channel)
    pub async fn get_current_progress(&self) -> ScanProgressMessage {
        if let Some(ref state) = self.scan_state {
            return state.to_progress_message();
        }
        // Fallback to broadcast channel if scan_state not set
        self.get_current_message().await
    }
}
```

**Initialization in `app.rs`**:
```rust
let mut broadcaster = Arc::new(ScanProgressBroadcaster::new());
let scan_state = Arc::new(ScanStateManager::new(broadcaster.sender()));

// Break circular dependency using Arc::make_mut()
Arc::make_mut(&mut broadcaster).set_scan_state(scan_state.clone());
```

This design allows:
- **WebSocket messages**: Sent via broadcast channel from `ScanStateManager` worker
- **HTTP API** (`/api/system/scan/progress`): Returns current state via `to_progress_message()`

### WebSocket Connection

- **Endpoint**: `WS /ws/scan`
- **Protocol**: Native WebSocket (not STOMP/SockJS like Java version)
- **Client**: `frontend/src/services/websocket.ts` - `ScanProgressWebSocketService` class
- **Reconnect**: Automatic reconnection every 5 seconds if connection lost

### Frontend Chinese Progress Text

The frontend generates Chinese display text locally based on the `phase` value.

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

### Frontend Scan Progress UI

**State Management** (`frontend/src/views/HomeView.vue`):

```typescript
// 刷新状态（控制按钮图标）
const refreshStatus = ref<'default' | 'refreshing' | 'success' | 'error'>('default')

// 扫描进度数据
const scanProgressData = ref<{
  status: 'progress' | 'completed' | 'error' | 'idle'
  phase?: string
  totalFiles: number
  successCount: number
  failureCount: number
  progressPercentage: string
  filesToAdd?: number
  filesToUpdate?: number
  filesToDelete?: number
}>({
  status: 'idle',
  totalFiles: 0,
  successCount: 0,
  failureCount: 0,
  progressPercentage: ''
})

// 弹窗显示控制
const showScanPopup = ref(false)
const scanPopupRef = ref<HTMLElement | null>(null)

// 可靠的是否正在扫描状态
const isScanning = computed(() => {
  const status = scanProgressData.value?.status
  return status === 'progress' || status === 'completed' || status === 'error'
})
```

**Button Click Logic** (`handleRefresh`):
```typescript
const handleRefresh = async () => {
  // 如果正在扫描，点击按钮切换弹窗显示
  if (isScanning.value) {
    showScanPopup.value = !showScanPopup.value
    return
  }

  // 不在扫描状态，点击按钮触发新扫描
  try {
    refreshStatus.value = 'refreshing'
    await systemApi.rescan()
  } catch (error) {
    refreshStatus.value = 'error'
    showScanPopup.value = false
  }
}
```

**WebSocket Progress Handler** (`handleScanProgress`):
```typescript
const handleScanProgress = (progress: ScanProgressMessage) => {
  if (progress.status === 'progress') {
    scanProgressData.value = {
      status: 'progress',
      phase: progress.phase,
      totalFiles: progress.totalFiles || 0,
      successCount: progress.successCount || 0,
      failureCount: progress.failureCount || 0,
      progressPercentage: progress.progressPercentage || '0',
      filesToAdd: progress.filesToAdd || 0,
      filesToUpdate: progress.filesToUpdate || 0,
      filesToDelete: progress.filesToDelete || 0
    }
    return
  }

  switch (progress.status) {
    case 'completed':
      showScanPopup.value = false
      refreshStatus.value = 'success'
      // 2秒后恢复默认状态
      setTimeout(() => { refreshStatus.value = 'default' }, 2000)
      break
    case 'error':
      refreshStatus.value = 'error'
      // 3秒后恢复默认状态
      setTimeout(() => { refreshStatus.value = 'default' }, 3000)
      break
    case 'cancelled':
      refreshStatus.value = 'default'
      break
  }
}
```

**Popup Display Conditions**:

| Condition | Desktop | Mobile |
|-----------|---------|--------|
| Show popup | `showScanPopup && !isMobile` | `showScanPopup && isMobile` |
| Popup position | Absolute, below button | Fixed, bottom sheet |
| Close on outside click | Yes | Yes (mask click) |
| Auto-close on complete | Yes | Yes |

**UI Interaction Flow**:
```
User clicks refresh button
         │
         ├── isScanning === true
         │         │
         │         └── Toggle showScanPopup → Show/hide popup
         │
         └── isScanning === false
                   │
                   └── Call rescan() → Show refreshing icon
                            │
                            └── Receive WebSocket progress
                                      │
                                      └── Update scanProgressData
```

### WebSocket Service (`frontend/src/services/websocket.ts`)

**ScanProgressWebSocketService** provides real-time scan progress updates:

| Method | Purpose |
|--------|---------|
| `connect()` | Connect to `/ws/scan`, auto-reconnect on disconnect |
| `disconnect()` | Close WebSocket connection |
| `onProgress(callback)` | Register progress callback |
| `offProgress()` | Remove progress callback |
| `isReady()` | Check connection status |

**Reconnection Logic**:
- Automatic reconnection every 5 seconds if connection lost
- Console logging for debugging

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
| Streaming | bytes 1, tokio-util 0.7 (for `ReaderStream`) |
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

### PhotoViewer Component - Metadata Display

The `PhotoViewer.vue` component displays photo metadata in detail view. Key features:

**Displayed Metadata Fields**:
| Category | Fields |
|----------|--------|
| Time | `exifTimestamp`, `createTime`, `modifyTime` |
| Device | `cameraMake`, `cameraModel`, `lensModel` |
| Settings | `exposureTime`, `aperture`, `iso`, `focalLength` |
| File | `width`, `height`, `fileSize`, `duration`, `videoCodec` |

**UI Layout**:
- Metadata organized into groups (Time, Device, Settings, File)
- Group titles removed, only fields displayed
- Fields within groups displayed as inline-blocks
- Each field shows label and value on separate lines

**Formatting**:
```typescript
// Exposure time (e.g., "1/125" → "1/125.000s", precision to 3 decimal places)
const formatExposureTime = (exposureTime: string) => {
  if (exposureTime.startsWith('1/')) {
    const denominator = parseFloat(exposureTime.substring(2))
    if (!isNaN(denominator)) {
      return `1/${denominator.toFixed(3)}s`
    }
  }
  return `${exposureTime}s`
}
```

**Frontend Type Definition** (`frontend/src/types/index.ts`):
```typescript
export interface MediaFile {
  id: string
  fileName: string
  fileType: 'image' | 'video'
  mimeType?: string
  fileSize?: number
  width?: number
  height?: number
  exifTimestamp?: string
  exifTimezoneOffset?: string
  createTime?: string
  modifyTime?: string
  cameraMake?: string
  cameraModel?: string
  lensModel?: string
  exposureTime?: string
  aperture?: string
  iso?: number
  focalLength?: string
  duration?: number
  videoCodec?: string
}
```

## Guidelines

Before taking ANY motivating change, understand the following guidelines first.

### Common Development Guideline

 - When modifing any existing code that could infect any existing logic or behavior, do not ignore any potential side effect. MAKE SURE  to check all the relevant code, or serious bugs might be introduced.
 - After any massive code change, e.g. feature and refactor, or critical logical change & breaking change, MAKE SURE to check and update the logical design to CLAUDE.md for future reference.
 - If you have multiple approach for a hard or systematic problem, programming an rust example to test if it could work in the first place is a good idea.

### Rust Development Guideline

Before using any crates, make sure to accomplish the following steps:

1. Update `Cargo.toml` with the required crate versions. If not sure about the version, do web search, or stop and call the question tool to ask me.
2. Check the crate documentation. First MAKE SURE to run `cargo doc` to build the documentation locally. Then read the documentation under `target/doc/` for the crate you are using. If the document is not accessible, try to read the source code directly.
3. After you are all clear about the crate's functionality, definition and usage associated with your goal, you can proceed with coding. 

Remember, just starting developing the code without reading the documentation may lead to unexpected errors or bugs, which is NOT ACCPETABLE. Rust has a decent documentation system, make nice use of it.

### Debug Guideline

 - Debugging must be wide-range, detailed and cautious. Never omit any relevant information and suspicious code when debugging.
 - For those difficult-to-debug issues, giving arbitrary conclusions for the problem is COMPLETELY unacceptable. 
   - You should be clear that you are an AI assistant and lack of outer information, might make mistakes and wrong decisions from time to time. 
   - Give final conclusions only when you are totally sure you do build up a completed, trusted evidence chain towards the reason. 
 - Feel free to ask me the user for more information or necessary manual operation if needed.
 - Do not hesitate to add well-detailed debug information, e.g. error messages, stack traces, or any other relevant details when lack of information. 
 
 ### Frontend Design Guideline

  - The design principle of the frontend is to keep it elegant, simple to use, and match with the human intuition. 
  - By default the UI should not contain too much bloated information, only show detail when user open the detail view. The detail view should be well organized and easy to read, and the entry is easy to find.

 ### Git Commit Guideline

  - DO NOT COMMIT ALL the files. Only commit the files modified in the current conversation.
  - When doing major changes, ONLY commit when I comfirm the program works successfully as expected.
  - The commit message should be clear and concise, in several lines of human-style sentences.

## Known Issues

### iPhone mov 视频 Seek 失败

**问题描述**：
- iPhone 拍摄的 mov 格式视频，点击进度条跳转到中间/末尾位置时会出现"视频播放失败"错误
- mp4 格式视频正常

**根本原因**：
- iPhone mov 文件的 moov atom 位于文件末尾（这是 HEVC/H.265 编码的常见做法）
- 使用 HTTP Range 请求进行流式播放时，seek 到中间/末尾返回的数据不包含 moov 元数据
- 浏览器无法解码，返回 `MEDIA_ERR_SRC_NOT_SUPPORTED` 错误

**后续修复方向**：
- 为 mov 格式使用完整文件请求（而非 Range 请求）
- 或在服务端检测 moov atom 位置，确保返回包含 moov 的数据

### 时区处理策略

**设计原则**：
1. **前端显示**：直接显示时间字面量，不进行时区转换
2. **后端排序**：使用时间字面量排序（不转换为 UTC）
3. **时区提示**：仅当照片时区与用户本地时区不同时，在时间后添加照片拍摄时区标签

**实现方式** (`PhotoViewer.vue:formatDate`)：
```typescript
const formatDate = (dateString: string, timezoneOffset?: string) => {
  const date = new Date(dateString)

  if (!timezoneOffset) {
    return `${date.toLocaleString('zh-CN')}`
  }

  // 解析时区偏移量
  const offsetHours = parseInt(timezoneOffset.substring(1, 3))
  const offsetMinutes = parseInt(timezoneOffset.substring(4, 6))
  const offsetSign = timezoneOffset[0] === '+' ? 1 : -1
  const totalOffsetMinutes = offsetSign * (offsetHours * 60 + offsetMinutes)

  // 检查是否与用户本地时区一致
  const userOffset = date.getTimezoneOffset()
  const isSameTimezone = userOffset === -totalOffsetMinutes

  if (isSameTimezone) {
    // 时区一致：直接显示时间
    return date.toLocaleString('zh-CN')
  } else {
    // 时区不同：显示时间并标注照片时区
    const timezoneLabel = `UTC${timezoneOffset}`
    return `${date.toLocaleString('zh-CN')} (${timezoneLabel})`
  }
}
```

**已知问题**：不同时区拍摄的照片可能按时间顺序错乱。由于采用"时间字面量"策略，2024-06-15 10:00:00 UTC+8 的照片可能排在 2024-06-15 02:00:00 UTC+0 的照片后面（按字面量 02:00 < 10:00），尽管前者实际上拍摄时间更晚。暂不修复。
