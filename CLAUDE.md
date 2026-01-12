# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Latte Album is a personal photo album application for NAS deployment. It consists of a Spring Boot 3.2 backend (Java 17) with a Vue 3 + TypeScript frontend (Element Plus).

## Build Commands

**Backend (Maven)**:
```bash
./dev-run.sh                        # 开发模式运行（使用 .env.development）
mvn clean package                   # Build JAR
mvn spring-boot:run                 # Run directly (需要配置 .env)
java -jar target/latte-album-*.jar  # Run JAR
```

**Frontend (npm)**:
```bash
cd frontend
npm install                         # Install dependencies
npm run dev                         # Dev server (port 3000)
npm run build                       # Production build (type-checked)
```

## Test Commands

```bash
mvn test                             # All tests
mvn test -Dtest=ClassName            # Specific test class
```

## Architecture

### Backend Structure
- `controller/` - REST API endpoints (prefix: `/album/api`)
- `service/` - Business logic
- `repository/` - Data access (JPA/SQLite)
- `model/` - JPA entities
- `dto/` - Request/response objects
- `util/` - Utility classes (TimeUtils)
- `config/` - Spring configuration
- `task/` - Scheduled tasks

### Key Services
| Service | Responsibility |
|---------|---------------|
| `FileScannerService` | Scans directories, extracts EXIF, generates thumbnails |
| `MediaProcessorService` | Image/video processing, metadata extraction, thumbnail generation |
| `MediaFileService` | Query, filter, paginate media files |
| `CacheService` | Two-level caching (Caffeine memory + disk) |
| `HeifProcessorService` | Async HEIF/HEIC conversion |
| `VideoProcessorService` | Video metadata extraction and thumbnail generation |
| `ScanProgressWebSocketService` | Real-time scan progress broadcasting |
| `TimeUtils` | Time validation and effective sort time calculation |

### Async Thread Pools (AsyncConfig)
| Executor | Core Pool | Max Pool | Queue | Purpose |
|----------|-----------|----------|-------|---------|
| `thumbnailGenerationExecutor` | CPU | CPU×2 | 200 | CPU-intensive thumbnail generation |
| `heifConversionExecutor` | max(4, CPU) | CPU×3 | 100 | IO-intensive HEIF conversion |
| `fileScanExecutor` | CPU | CPU×2 | 500 | Parallel file scanning |
| `videoProcessingExecutor` | CPU | CPU×2 | 50 | Video processing |

All use `CallerRunsPolicy` (rejected tasks run in caller thread).

### API Endpoints

**File Operations**:
- `GET /api/files` - List with pagination, sorting, filtering
- `GET /api/files/dates` - Get dates with photos (calendar indicator)
- `GET /api/files/{id}` - File details
- `GET /api/files/{id}/thumbnail?size={small|medium|large}` - Thumbnail stream
- `GET /api/files/{id}/original` - Original file stream
- `GET /api/files/{id}/neighbors` - Prev/next for navigation
- `GET /api/directories` - Directory tree

**System Operations**:
- `POST /api/system/rescan` - Trigger directory rescan
- `GET /api/system/status` - System status
- `GET /api/system/scan/progress` - Scan progress (HTTP fallback)
- `WS /ws/scan` - WebSocket for real-time scan progress

### Frontend Structure
- `views/` - Page components (HomeView)
- `components/` - Reusable Vue components (PhotoViewer, DateNavigator, MediaCard, Gallery)
- `stores/` - Pinia stores (gallery.ts)
- `services/` - API client (api.ts, websocket.ts)
- `types/` - TypeScript interfaces

### Configuration

Key configuration in `application.yml`:
```yaml
album:
  base-path: ${ALBUM_BASE_PATH:./photos}        # Photo directory
  scan:
    cron: "0 0 2 * * ?"                         # Scheduled rescan (2 AM)
    parallel:
      enabled: true                             # Parallel scanning
      thread-pool-size: 0                       # Auto: CPU cores
      batch-size: 50                            # Batch save size
  thumbnail:
    small: 300                                  # Small thumbnail (px)
    medium: 450                                 # Medium thumbnail (px)
    large: 900                                  # Large thumbnail (px)
    quality: 0.8                                # JPEG quality (80%)
  cache:
    enabled: true
    disk-path: ${ALBUM_CACHE_DIR:./cache}
  video:
    ffmpeg-path: /usr/bin/ffmpeg
    thumbnail-time-offset: 1.0                  # Thumbnail offset (seconds)
    thumbnail-duration: 0.1                     # Thumbnail duration (seconds)
```

Environment variables: `.env.development` for development, `.env` for production.

## Database

SQLite with Hibernate JPA. Key tables:
- `media_files` - Media file metadata and EXIF data
- `directories` - Scanned directory records
- `system_config` - Configuration storage

## Supported File Formats

**Images**: `.jpg`, `.jpeg`, `.png`, `.gif`, `.bmp`, `.webp`, `.tiff`, `.heif`, `.heic`

**Videos**: `.mp4`, `.avi`, `.mov`, `.mkv`, `.wmv`, `.flv`, `.webm`

## Key Implementation Details

### Time Sorting Logic

Priority order for sorting:
1. **EXIF timestamp** (`exifTimestamp`) - Primary, from photo metadata
2. **File create time** (`createTime`) - Fallback
3. **File modify time** (`modifyTime`) - Last resort

Uses `TimeUtils.getEffectiveSortTime()` for unified sorting across different timestamp sources. The `sortBy=exifTimestamp` parameter uses composite SQL sorting to handle files with/without EXIF data together (not split into groups).

**Time Validation** (`TimeUtils`):
- Valid year range: 1900 - current year + 1
- Create time cannot be in the future
- EXIF time can be in future (timezone handling预留)

### File Scanning

**Scanning Modes**:
- **Serial**: Default, processes files sequentially
- **Parallel**: Uses `@Async("fileScanExecutor")` for concurrent processing

**Scanning Process**:
1. First startup: Automatic full scan if database is empty
2. Recursively traverse directories
3. Compare file modification time with `lastScanned`
4. Incremental update: Only reprocess modified files
5. Delete database records for missing files

**Incremental Scan Logic**:
```java
if (mediaFile.getLastScanned() == null ||
    mediaFile.getLastScanned().isBefore(lastModified)) {
    // Reprocess file
}
```

**Real-time Progress**:
- WebSocket broadcasts to `/topic/scan/progress`
- Messages: `started`, `progress`, `completed`, `error`, `cancelled`
- Frontend displays circular progress indicator on refresh button

### Metadata Extraction

**EXIF Fields** (using `metadata-extractor` library):
| Field | EXIF Tag | Description |
|-------|----------|-------------|
| `cameraMake` | TAG_MAKE | Camera manufacturer |
| `cameraModel` | TAG_MODEL | Camera model |
| `exifTimestamp` | TAG_DATETIME_ORIGINAL | Photo capture time |
| `exifTimezoneOffset` | 0x9010 | Timezone offset |
| `aperture` | TAG_FNUM | Aperture value |
| `exposureTime` | TAG_EXPOSURE_TIME | Shutter speed |
| `iso` | TAG_ISO_EQUIVALENT | ISO sensitivity |
| `focalLength` | TAG_FOCAL_LENGTH | Focal length |

**Time Storage**:
- Stored in UTC to avoid timezone conversion issues
- Original timezone offset preserved in `exifTimezoneOffset`

### Thumbnail Generation

**Process Flow**:
1. Detect file type (image/video/HEIF)
2. Video: Use `VideoProcessorService.extractThumbnail()`
3. HEIF: Convert to JPEG first via `HeifProcessorService`, then resize
4. Regular images: Use `thumbnailator` library directly

**Thumbnail Sizes**:
| Size | Pixels | Use Case |
|------|--------|----------|
| small | 300px | Grid view thumbnails |
| medium | 450px | Standard display |
| large | 900px | Full-screen viewing |

**Cache Keys**: `{fileId}_{size}.jpg`

### HEIF/HEIC Support

**Formats**: `.heic`, `.heif` (case-insensitive)

**Conversion**:
- Uses external tools: `heif-convert`, `heif-info` (from libheif)
- Converts to JPEG for thumbnails and display
- Extracts dimensions via `heif-info` command

**Async Processing**:
- Uses `@Async("heifConversionExecutor")`
- Fallback: Serve original file if conversion unavailable

**Method Examples**:
```java
// Synchronous conversion
convertToJpeg(File input, File output)

// Async conversion with quality control
convertHeif(File input, File output, String format, Integer quality)

// Direct thumbnail generation
generateThumbnail(File input, int targetWidth)
```

### Video Processing

**Library**: `jave` (Java Audio Video Encoder)

**Extracted Metadata**:
- Duration (seconds)
- Resolution (width × height)
- Video codec (H.264, H.265, etc.)

**Thumbnail Generation**:
- Encoder: MJPEG
- Time offset: `videoThumbnailTimeOffset` (default: 1.0s)
- Duration: `videoThumbnailDuration` (default: 0.1s)
- Frame rate: 1 fps

### Caching Strategy

**Three-Level Cache Architecture**:

| Level | Component | TTL | Capacity |
|-------|-----------|-----|----------|
| L1 | Caffeine Memory | 1 hour | 1000 entries |
| L2 | Disk Cache | Permanent | All thumbnails |
| L3 | Database | Permanent | EXIF metadata |

**Cache Flow**:
1. Check memory cache first
2. Fall back to disk cache
3. Generate on miss, then cache

### Filtering and Query

**Query Parameters** (`GET /api/files`):
| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | String | Directory path filter |
| `page` | Integer | Page number (default: 0) |
| `size` | Integer | Page size (default: 50) |
| `sortBy` | String | Sort field: `exifTimestamp`, `createTime`, `modifyTime`, `fileName` |
| `order` | String | `asc` or `desc` (default: desc) |
| `filterType` | String | `all`, `image`, or `video` |
| `cameraModel` | String | Camera model filter |
| `date` | String | Date filter (YYYY-MM-DD) |

**Date Query** (`GET /api/files/dates`):
- Returns list of dates containing photos
- Supports same filtering as main query
- Used for calendar highlighting

### Neighbor Navigation

**Endpoint**: `GET /api/files/{id}/neighbors`

Uses `TimeUtils.getEffectiveSortTime()` to find:
- Previous file (chronologically before)
- Next file (chronologically after)

Enables swipe navigation in PhotoViewer.

### WebSocket Progress Broadcasting

**Endpoint**: `WS /ws/scan` (via SockJS)

**Topic**: `/topic/scan/progress`

**Message Types**:
```typescript
interface ScanProgressMessage {
  scanning: boolean
  totalFiles: number
  successCount: number
  failureCount: number
  progressPercentage: string  // e.g., "45.23"
  status: 'started' | 'progress' | 'completed' | 'error' | 'cancelled'
  message?: string
}
```

**Frontend Integration**:
- Auto-connect on HomeView mount
- Circular progress indicator on refresh button
- Auto-reconnect on disconnect

### MediaFile Entity

```java
@Entity
public class MediaFile {
    Long id;
    String filePath;              // Unique
    String fileName;
    String fileType;              // "image" or "video"
    String mimeType;
    Long fileSize;                // Bytes

    // Dimensions
    Integer width;
    Integer height;

    // Time fields
    LocalDateTime exifTimestamp;  // EXIF capture time
    String exifTimezoneOffset;    // e.g., "+08:00"
    LocalDateTime createTime;     // File creation
    LocalDateTime modifyTime;     // Last modified
    LocalDateTime lastScanned;    // Last scan time

    // Camera info
    String cameraMake;
    String cameraModel;

    // Exposure settings
    String aperture;              // e.g., "f/2.8"
    String exposureTime;          // e.g., "1/1000s"
    Integer iso;
    String focalLength;           // e.g., "50mm"

    // Video specific
    Double duration;              // Seconds
    String videoCodec;

    // Processing
    Boolean thumbnailGenerated;
}
```

### Development Notes

- **Vue Query Client**: Use `useQuery` from `@tanstack/vue-query` for data fetching
- **Type Safety**: All DTOs have TypeScript equivalents in `frontend/src/types/`
- **Hot Reload**: Frontend supports HMR; restart required for backend changes
- **Database**: SQLite file location configured via `ALBUM_DB_PATH` env var
- **Logs**: Backend logs via SLF4J (`FileScannerService` uses `logger`)

### Common Tasks

**Add new file format support**:
1. Add extension to `IMAGE_EXTENSIONS` or `VIDEO_EXTENSIONS` in `FileScannerService`
2. Add processing logic in `MediaProcessorService.processImage/processVideo`
3. Update `SUPPORTED_FORMATS` in frontend if needed

**Modify thumbnail sizes**:
1. Update `album.thumbnail.{small|medium|large}` in `application.yml`
2. Update `ThumbnailSize` enum in frontend if adding new sizes

**Add new EXIF field**:
1. Add field to `MediaFile` entity
2. Extract in `MediaProcessorService.extractExifMetadata()`
3. Add to TypeScript interface in `frontend/src/types/index.ts`
4. Update frontend display components

## Project Rules

These rules must be followed when making changes to this codebase:

1. **Code Structure**: Keep code simple and clear. Avoid over-abstraction.
2. **Single Responsibility**: Each component/class should have a single, clear responsibility. Avoid feature coupling.
3. **Code Reuse**: Consolidate duplicate code. DRY principle.
4. **Comments**: Add clear, readable comments for complex key logic.
5. **Testing**: Ensure unit test coverage for all functional additions and changes.
6. **Cleanup**: Remove obsolete code promptly during refactoring and development.
7. **Privacy**: Do not directly access production databases, images, or image-related APIs unless explicitly authorized. Instead, provide steps for manual confirmation.

## Codebase File Reference

### Backend - Controller Layer

| File | Responsibility |
|------|----------------|
| `FilesController.java` | Media file REST API: pagination, sorting, filtering, thumbnails, neighbors, dates |
| `SystemController.java` | System operations: rescan trigger, progress query, cancel, status |
| `DirectoriesController.java` | Directory tree retrieval |

### Backend - Service Layer

| File | Responsibility |
|------|----------------|
| `MediaFileService.java` | Query logic with filters, effective time sorting, neighbor navigation |
| `FileScannerService.java` | Directory scanning, EXIF extraction, thumbnail generation, incremental update |
| `MediaProcessorService.java` | Image/video processing entry, EXIF extraction, thumbnail orchestration |
| `CacheService.java` | Three-level cache (memory → disk → generate) for thumbnails |
| `HeifProcessorService.java` | HEIF/HEIC to JPEG conversion, async thumbnail generation |
| `VideoProcessorService.java` | Video metadata extraction, preview thumbnail generation |
| `ScanProgressWebSocketService.java` | Real-time progress broadcasting via WebSocket/STOMP |

### Backend - Repository Layer

| File | Responsibility |
|------|----------------|
| `MediaFileRepository.java` | MediaFile CRUD, effective time queries, neighbor navigation, date queries |
| `DirectoryRepository.java` | Directory persistence |

### Backend - Model Layer

| File | Responsibility |
|------|----------------|
| `MediaFile.java` | JPA entity: all media metadata, EXIF, video fields |
| `Directory.java` | JPA entity: scanned directory records |

### Backend - DTO/Mapper/Config

| File | Responsibility |
|------|----------------|
| `MediaFileDTO.java` | API response transfer object |
| `DateDTO.java` | Date with photo count |
| `ScanProgressDTO.java` | WebSocket progress messages |
| `MediaFileMapper.java` | Entity ↔ DTO conversion |
| `ApplicationConfig.java` | Application configuration properties |
| `AsyncConfig.java` | Thread pool configuration (4 executors) |
| `WebSocketConfig.java` | STOMP WebSocket broker configuration |

### Backend - Util/Task

| File | Responsibility |
|------|----------------|
| `TimeUtils.java` | Time validation, effective sort time calculation |
| `ScheduledTasks.java` | Daily 2 AM automatic scan |

### Frontend - Views

| File | Responsibility |
|------|----------------|
| `HomeView.vue` | Main page: Gallery + DateNavigator + PhotoViewer, refresh with WebSocket progress |

### Frontend - Components

| File | Responsibility |
|------|----------------|
| `Gallery.vue` | Masonry layout gallery, responsive columns (2-4), infinite scroll |
| `MediaCard.vue` | Thumbnail display, video duration badge, lazy loading |
| `PhotoViewer.vue` | Fullscreen viewer, navigation, EXIF display, download |
| `DateNavigator.vue` | Calendar picker, date navigation, disable dates without photos |

### Frontend - Stores/Services

| File | Responsibility |
|------|----------------|
| `gallery.ts` | Pinia store: items, pagination, sorting/filtering state |
| `api.ts` | Axios client: file, directory, system APIs |
| `websocket.ts` | STOMP client: scan progress subscription, auto-reconnect |

### Frontend - Types

| File | Responsibility |
|------|----------------|
| `index.ts` | TypeScript interfaces: MediaFile, DateInfo, Directory, PaginatedResponse |
