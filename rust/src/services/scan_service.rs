use crate::config::Config;
use crate::db::{DatabasePool, MediaFile, MediaFileRepository};
use crate::processors::ProcessorRegistry;
use crate::services::CacheService;
use crate::websocket::ScanProgressBroadcaster;
use futures_util::future::try_join_all;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs;

/// Scan progress tracking
#[derive(Debug, Clone, Default)]
pub struct ScanProgress {
    pub scanning: bool,
    pub phase: Option<String>,
    pub phase_message: Option<String>,
    pub total_files: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub files_to_add: u64,
    pub files_to_update: u64,
    pub files_to_delete: u64,
}

impl ScanProgress {
    pub fn get_progress_percentage(&self) -> String {
        if self.total_files == 0 {
            return "0.00".to_string();
        }
        let percentage = (self.success_count + self.failure_count) as f64 / self.total_files as f64 * 100.0;
        format!("{:.2}", percentage)
    }
}

/// Service for scanning media files
pub struct ScanService {
    config: Config,
    db: DatabasePool,
    processors: Arc<ProcessorRegistry>,
    cache: Arc<CacheService>,
    broadcaster: Arc<ScanProgressBroadcaster>,

    // Scan state
    is_scanning: Arc<AtomicBool>,
    is_cancelled: Arc<AtomicBool>,
    total_files: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
    failure_count: Arc<AtomicU64>,
}

impl ScanService {
    pub fn new(
        config: Config,
        db: DatabasePool,
        processors: Arc<ProcessorRegistry>,
        cache: Arc<CacheService>,
        broadcaster: Arc<ScanProgressBroadcaster>,
    ) -> Self {
        Self {
            config,
            db,
            processors,
            cache,
            broadcaster,
            is_scanning: Arc::new(AtomicBool::new(false)),
            is_cancelled: Arc::new(AtomicBool::new(false)),
            total_files: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            failure_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Start a scan operation
    pub async fn scan(&self, parallel: bool) {
        tracing::info!("Scanning media files with parallel: {}", parallel);
        if self.is_scanning.load(Ordering::SeqCst) {
            tracing::warn!("Scan already in progress");
            return;
        }

        self.is_scanning.store(true, Ordering::SeqCst);
        self.is_cancelled.store(false, Ordering::SeqCst);
        self.total_files.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);

        self.perform_scan(parallel).await;

        self.is_scanning.store(false, Ordering::SeqCst);

        // Broadcast completion
        self.broadcaster.send_completed().await;
    }

    async fn perform_scan(&self, parallel: bool) {
        tracing::info!("Starting scan with parallel: {}", parallel);
        // Phase 1: Collect all files
        self.broadcaster.send_started().await;

        let files = match self.collect_files().await {
            Ok(files) => files,
            Err(e) => {
                tracing::error!("Failed to collect files: {}", e);
                return;
            }
        };

        let total = files.len() as u64;
        self.total_files.store(total, Ordering::SeqCst);
        self.broadcaster.update_total(total).await;

        if total == 0 {
            self.broadcaster.send_completed().await;
            return;
        }

        // Phase 2: Calculate changes
        let counts = self.calculate_changes(&files).await;
        self.broadcaster.update_phase("processing", &format!(
            "Found {} to add, {} to update, {} to delete",
            counts.files_to_add, counts.files_to_update, counts.files_to_delete
        )).await;

        // Phase 3: Process files
        if parallel {
            self.process_parallel(&files).await;
        } else {
            self.process_serial(&files).await;
        }

        // Phase 4: Clean up missing files
        self.delete_missing(&files).await;
    }

    async fn collect_files(&self) -> std::io::Result<Vec<PathBuf>> {
        self.broadcaster.update_phase("collecting", "Scanning directory...").await;

        let mut files = Vec::new();
        let base_path = &self.config.base_path;

        tracing::info!("Scanning directory: {:?}", base_path);

        // Check if base path exists
        if !base_path.exists() {
            tracing::error!("Base path does not exist: {:?}", base_path);
            return Ok(files);
        }

        if !base_path.is_dir() {
            tracing::error!("Base path is not a directory: {:?}", base_path);
            return Ok(files);
        }

        // Supported extensions
        let supported_extensions = [
            "jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "heic", "heif",
            "mp4", "avi", "mov", "mkv", "wmv", "flv", "webm"
        ];

        // Walk directory recursively
        let mut stack = vec![base_path.clone()];

        while let Some(current_dir) = stack.pop() {
            if self.is_cancelled.load(Ordering::SeqCst) {
                break;
            }

            match fs::read_dir(&current_dir).await {
                Ok(mut entries) => {
                    while let Some(entry) = entries.next_entry().await? {
                        let path = entry.path();

                        if path.is_file() {
                            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                if supported_extensions.contains(&ext.to_lowercase().as_str()) {
                                    files.push(path);
                                }
                            }
                        } else if path.is_dir() {
                            stack.push(path);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to read directory {:?}: {}", current_dir, e);
                }
            }
        }

        tracing::info!("Scan complete: {} files found", files.len());
        Ok(files)
    }

    async fn calculate_changes(&self, files: &[PathBuf]) -> ScanProgress {
        let repo = MediaFileRepository::new(&self.db);

        // Count new, updated, and deleted files
        let mut to_add = 0;
        let mut to_update = 0;

        for path in files {
            match repo.find_by_path(path).await {
                Ok(Some(existing)) => {
                    // Check if file was modified
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Some(last_scanned) = existing.last_scanned {
                                // Simple comparison - would need proper time comparison
                                to_update += 1;
                            } else {
                                to_update += 1;
                            }
                        } else {
                            to_update += 1;
                        }
                    } else {
                        to_update += 1;
                    }
                }
                Ok(None) => {
                    to_add += 1;
                }
                Err(_) => {
                    to_add += 1;
                }
            }
        }

        ScanProgress {
            files_to_add: to_add,
            files_to_update: to_update,
            files_to_delete: 0,
            ..Default::default()
        }
    }

    async fn process_serial(&self, files: &[PathBuf]) {
        let total = files.len() as u64;

        for (index, file) in files.iter().enumerate() {
            if self.is_cancelled.load(Ordering::SeqCst) {
                self.broadcaster.send_cancelled().await;
                return;
            }

            if index % 10 == 0 {
                self.broadcaster.update_progress(index as u64, total).await;
            }

            if let Err(e) = self.process_file(file).await {
                tracing::error!("Failed to process {}: {}", file.display(), e);
                self.failure_count.fetch_add(1, Ordering::SeqCst);
            } else {
                self.success_count.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    async fn process_parallel(&self, files: &[PathBuf]) {
        // Use tokio::task for parallel processing
        let total = files.len() as u64;

        // Process in batches to avoid overwhelming the system
        let batch_size = 10;
        let chunks: Vec<_> = files.chunks(batch_size).collect();

        for (batch_index, chunk) in chunks.iter().enumerate() {
            if self.is_cancelled.load(Ordering::SeqCst) {
                self.broadcaster.send_cancelled().await;
                return;
            }

            let progress = (batch_index * batch_size) as u64;
            self.broadcaster.update_progress(progress, total).await;

            // Process files in this batch sequentially
            let mut success_count = 0;
            let mut failure_count = 0;
            for file in chunk.iter() {
                match self.process_file(file).await {
                    Ok(_) => success_count += 1,
                    Err(_) => failure_count += 1,
                }
            }
            self.success_count.fetch_add(success_count, Ordering::SeqCst);
            self.failure_count.fetch_add(failure_count, Ordering::SeqCst);
        }
    }

    async fn process_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let repo = MediaFileRepository::new(&self.db);

        // Find processor for this file
        let processor = self.processors.find_processor(path).ok_or_else(|| {
            Box::new(std::io::Error::new(std::io::ErrorKind::Unsupported, "No processor found"))
        })?;

        // Extract file metadata first (file_size, create_time, modify_time - format independent)
        let file_metadata = crate::processors::file_metadata::extract_file_metadata(path);

        // Process file to get format-specific metadata
        let format_metadata = processor.process(path).await?;

        // Create or update media file record
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_type = if processor.media_type() == crate::processors::MediaType::Video {
            "video"
        } else {
            "image"
        };

        let mut media_file = MediaFile::new(
            path.to_string_lossy().to_string(),
            file_name,
            file_type.to_string(),
        );

        // Apply file metadata (from extract_file_metadata)
        media_file.file_size = file_metadata.file_size;
        media_file.create_time = file_metadata.create_time;
        media_file.modify_time = file_metadata.modify_time;

        // Apply format-specific metadata (from processor)
        // These override file metadata if present (for fields that exist in both)
        media_file.mime_type = format_metadata.mime_type;
        media_file.width = format_metadata.width;
        media_file.height = format_metadata.height;
        media_file.exif_timestamp = format_metadata.exif_timestamp;
        media_file.exif_timezone_offset = format_metadata.exif_timezone_offset;
        media_file.camera_make = format_metadata.camera_make;
        media_file.camera_model = format_metadata.camera_model;
        media_file.lens_model = format_metadata.lens_model;
        media_file.exposure_time = format_metadata.exposure_time;
        media_file.aperture = format_metadata.aperture;
        media_file.iso = format_metadata.iso;
        media_file.focal_length = format_metadata.focal_length;
        media_file.duration = format_metadata.duration;
        media_file.video_codec = format_metadata.video_codec;

        // Upsert to database
        repo.upsert(&media_file).await?;

        // Note: Thumbnail generation is done on-demand when thumbnail API is called
        // This improves initial scan performance significantly

        Ok(())
    }

    async fn delete_missing(&self, existing_files: &[PathBuf]) {
        let repo = MediaFileRepository::new(&self.db);

        let existing_paths: Vec<String> = existing_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        if let Ok(count) = repo.delete_missing(&existing_paths).await {
            tracing::info!("Deleted {} missing files", count);
        }
    }

    /// Cancel the current scan
    pub async fn cancel(&self) -> bool {
        if self.is_scanning.load(Ordering::SeqCst) {
            self.is_cancelled.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }
}
