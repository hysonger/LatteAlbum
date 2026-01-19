use crate::config::Config;
use crate::db::{DatabasePool, MediaFile, MediaFileRepository};
use crate::processors::{MediaMetadata, ProcessorRegistry};
use crate::websocket::{ScanStateManager, ScanPhase};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs;
use tokio::sync::Semaphore;

/// Scan progress tracking
#[derive(Debug, Clone, Default)]
pub struct ScanProgress {
    pub scanning: bool,
    pub phase: Option<String>,
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

/// Result of processing a single file
#[derive(Debug, Clone)]
struct ProcessingResult {
    path: PathBuf,
    success: Option<MediaFile>,
    error: Option<String>,
}

/// Service for scanning media files
pub struct ScanService {
    config: Config,
    db: DatabasePool,
    processors: Arc<ProcessorRegistry>,
    scan_state: Arc<ScanStateManager>,

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
        scan_state: Arc<ScanStateManager>,
    ) -> Self {
        Self {
            config,
            db,
            processors,
            scan_state,
            is_scanning: Arc::new(AtomicBool::new(false)),
            is_cancelled: Arc::new(AtomicBool::new(false)),
            total_files: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            failure_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get the concurrency level for parallel scanning
    fn get_concurrency(&self) -> usize {
        self.config.scan_concurrency.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|p| p.get() * 2)
                .unwrap_or(16)
        })
    }

    /// Start a scan operation
    pub async fn scan(&self, _parallel: bool) {
        tracing::info!("Scanning media files");
        if self.is_scanning.load(Ordering::SeqCst) {
            tracing::warn!("Scan already in progress");
            return;
        }

        self.is_scanning.store(true, Ordering::SeqCst);
        self.is_cancelled.store(false, Ordering::SeqCst);
        self.total_files.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);

        if self.config.scan_parallel {
            self.perform_scan_parallel().await;
        } else {
            self.perform_scan_serial().await;
        }

        self.is_scanning.store(false, Ordering::SeqCst);
    }

    /// Parallel scan implementation (default)
    async fn perform_scan_parallel(&self) {
        let scan_start = Instant::now();
        tracing::info!("Starting parallel scan");

        // 重置计数器，确保每次扫描从0开始
        self.scan_state.reset_counters();

        // Phase 1: Collect all file paths (fast, no DB access)
        // 在收集文件之前发送 Collecting 阶段，让前端立即看到扫描状态
        self.scan_state.set_phase(ScanPhase::Collecting);
        let collect_start = Instant::now();
        let files = match self.collect_file_paths().await {
            Ok(files) => files,
            Err(e) => {
                tracing::error!("Failed to collect files: {}", e);
                self.scan_state.error();
                return;
            }
        };
        let collect_duration = collect_start.elapsed();
        tracing::debug!("Phase 1 (collecting): {} files collected in {:?}", files.len(), collect_duration);

        let total = files.len() as u64;
        self.total_files.store(total, Ordering::SeqCst);
        self.scan_state.set_total(total);

        if total == 0 {
            // 设置完成状态
            self.scan_state.set_phase(ScanPhase::Completed);
            self.scan_state.completed();
            tracing::info!("Scan complete (no files) in {:?}", scan_start.elapsed());
            return;
        }

        // Phase 2: Batch check database for existing files
        let count_start = Instant::now();
        self.scan_state.set_phase(ScanPhase::Counting);
        let (files_to_add, files_to_update, skip_list) = self.batch_check_exists(&files).await;

        // Count files to delete
        let repo = MediaFileRepository::new(&self.db);
        let files_to_delete = match repo.count_missing(&files).await {
            Ok(count) => count,
            Err(e) => {
                tracing::warn!("Failed to count missing files: {}, assuming 0", e);
                0
            }
        };
        self.scan_state.set_file_counts(files_to_add, files_to_update, files_to_delete);

        let count_duration = count_start.elapsed();
        tracing::debug!("Phase 2 (counting): {} to add, {} to update, {} to skip, {} to delete in {:?}",
            files_to_add, files_to_update, skip_list.len(), files_to_delete, count_duration);

        let processing_count = files_to_add + files_to_update;
        if processing_count > 0 {
            self.scan_state.set_phase(ScanPhase::Processing);
            self.scan_state.set_total(processing_count);

            // Build list of files that need metadata extraction
            let mut files_to_process: Vec<PathBuf> = Vec::with_capacity(processing_count as usize);
            for path in &files {
                let path_str = path.to_string_lossy().to_string();
                if !skip_list.iter().any(|p| p.to_string_lossy().to_string() == path_str) {
                    files_to_process.push(path.clone());
                }
            }

            // Phase 3: Parallel metadata extraction (only for files that need it)
            let process_start = Instant::now();
            let results = self.parallel_extract_metadata(&files_to_process).await;
            let process_duration = process_start.elapsed();
            let success_results = results.iter().filter(|r| r.success.is_some()).count();
            let fail_results = results.iter().filter(|r| r.success.is_none()).count();
            tracing::debug!("Phase 3 (processing): {} processed ({} success, {} failed) in {:?}",
                results.len(), success_results, fail_results, process_duration);

            // Phase 4: Batch upsert results + update skip_list last_scanned
            self.scan_state.set_phase(ScanPhase::Writing);
            let writing_cancelled = self.batch_write_results_with_skip(results, &skip_list, total).await;
            let write_duration = process_start.elapsed();
            tracing::debug!("Phase 4 (writing): completed in {:?}", write_duration);

            // Check if writing was cancelled
            if writing_cancelled || self.is_cancelled.load(Ordering::SeqCst) {
                // 执行删除阶段（但删除操作内部会检查取消标志）
                self.scan_state.set_phase(ScanPhase::Deleting);
                self.delete_missing(&files).await;
                // 发送取消状态
                self.scan_state.cancelled();
                tracing::info!("Parallel scan cancelled after writing {} files", success_results);
                return;
            }
        } else {
            // All files unchanged - just update last_scanned for all
            self.scan_state.set_phase(ScanPhase::Writing);
            self.scan_state.set_file_counts(0, 0, files_to_delete);

            let write_start = Instant::now();
            let writing_cancelled = self.batch_write_results_with_skip(Vec::new(), &skip_list, total).await;
            let write_duration = write_start.elapsed();
            tracing::debug!("Phase 4 (updating): {} files touched in {:?}", skip_list.len(), write_duration);

            // Check if writing was cancelled
            if writing_cancelled || self.is_cancelled.load(Ordering::SeqCst) {
                self.scan_state.set_phase(ScanPhase::Deleting);
                self.delete_missing(&files).await;
                self.scan_state.cancelled();
                tracing::info!("Parallel scan cancelled during touch phase");
                return;
            }
        }

        // Phase 5: Clean up missing files
        self.scan_state.set_phase(ScanPhase::Deleting);
        self.delete_missing(&files).await;
        tracing::debug!("Phase 5 (deleting): completed");

        // Scan complete
        self.scan_state.completed();

        let processed = self.success_count.load(Ordering::SeqCst) + self.failure_count.load(Ordering::SeqCst);
        let total_duration = scan_start.elapsed();
        tracing::info!("Parallel scan complete: {} files processed ({} success, {} failed), {} unchanged skipped, total time: {:?}",
            processed, self.success_count.load(Ordering::SeqCst), self.failure_count.load(Ordering::SeqCst), skip_list.len(), total_duration);
    }

    /// Serial scan implementation (fallback)
    async fn perform_scan_serial(&self) {
        let scan_start = Instant::now();
        tracing::info!("Starting serial scan (fallback mode)");

        // 重置计数器，确保每次扫描从0开始
        self.scan_state.reset_counters();

        // Phase 1: Collect all file paths
        let collect_start = Instant::now();
        self.scan_state.set_phase(ScanPhase::Collecting);
        let files = match self.collect_file_paths().await {
            Ok(files) => files,
            Err(e) => {
                tracing::error!("Failed to collect files: {}", e);
                self.scan_state.error();
                return;
            }
        };
        let collect_duration = collect_start.elapsed();
        tracing::debug!("Phase 1 (collecting): {} files collected in {:?}", files.len(), collect_duration);

        let total = files.len() as u64;
        self.scan_state.set_total(total);

        if total == 0 {
            // 设置完成状态
            self.scan_state.set_phase(ScanPhase::Completed);
            self.scan_state.completed();
            tracing::info!("Scan complete (no files) in {:?}", scan_start.elapsed());
            return;
        }

        // Phase 2: Count changes
        let count_start = Instant::now();
        let counts = self.calculate_changes(&files).await;
        let count_duration = count_start.elapsed();
        tracing::debug!("Phase 2 (counting): {} to add, {} to update in {:?}",
            counts.files_to_add, counts.files_to_update, count_duration);

        // Set file counts and prepare for processing
        self.scan_state.set_file_counts(counts.files_to_add, counts.files_to_update, counts.files_to_delete);

        // Update to processing phase
        self.scan_state.set_phase(ScanPhase::Processing);
        self.scan_state.set_total(counts.files_to_add + counts.files_to_update);

        // Phase 3: Process files serially
        let process_start = Instant::now();
        let processing_count = counts.files_to_add + counts.files_to_update;
        if processing_count > 0 {
            // Process files that need metadata extraction
            self.process_serial(&files).await;
            // 检查是否在处理过程中被取消（process_serial 内部会调用 cancelled()）
            if self.is_cancelled.load(Ordering::SeqCst) {
                // 删除阶段会检查取消标志，这里仍然执行删除
                self.scan_state.set_phase(ScanPhase::Deleting);
                self.delete_missing(&files).await;
                tracing::info!("Serial scan cancelled");
                return;
            }
        } else {
            // All files unchanged - just touch them in batch
            let repo = MediaFileRepository::new(&self.db);
            let _ = repo.batch_touch(&files).await;
        }
        let process_duration = process_start.elapsed();
        tracing::debug!("Phase 3 (processing): completed in {:?}", process_duration);

        // Phase 4: Clean up missing files
        let delete_start = Instant::now();
        self.delete_missing(&files).await;
        let delete_duration = delete_start.elapsed();
        tracing::debug!("Phase 4 (deleting): completed in {:?}", delete_duration);

        let processed = self.success_count.load(Ordering::SeqCst) + self.failure_count.load(Ordering::SeqCst);
        let total_duration = scan_start.elapsed();
        tracing::info!("Serial scan complete: {} files processed ({} success, {} failed), total time: {:?}",
            processed, self.success_count.load(Ordering::SeqCst), self.failure_count.load(Ordering::SeqCst), total_duration);
    }

    /// Collect file paths only (fast operation)
    async fn collect_file_paths(&self) -> std::io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let base_path = &self.config.base_path;

        tracing::info!("Scanning directory: {:?}", base_path);

        if !base_path.exists() {
            tracing::error!("Base path does not exist: {:?}", base_path);
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Directory not found: {:?}", base_path)
            ));
        }

        if !base_path.is_dir() {
            tracing::error!("Base path is not a directory: {:?}", base_path);
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                format!("Not a directory: {:?}", base_path)
            ));
        }

        // Supported extensions
        let supported_extensions = [
            "jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "heic", "heif",
            "mp4", "avi", "mov", "mkv", "wmv", "flv", "webm"
        ];

        // Walk directory recursively using async stack (non-blocking)
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

        tracing::info!("Collected {} files", files.len());
        Ok(files)
    }

    /// Batch check which files exist in database (optimized for bulk queries)
    /// Returns (to_add, to_update, skip_list) - skip_list contains files with unchanged modify_time
    /// Uses batch_find_by_paths_batch for efficient bulk SELECT queries
    async fn batch_check_exists(&self, files: &[PathBuf]) -> (u64, u64, Vec<PathBuf>) {
        let batch_size = self.config.db_batch_check_size;

        let mut to_add = 0u64;
        let mut to_update = 0u64;
        let mut skip_list: Vec<PathBuf> = Vec::new();
        let repo = MediaFileRepository::new(&self.db);

        for chunk in files.chunks(batch_size) {
            if self.is_cancelled.load(Ordering::SeqCst) {
                break;
            }

            // Use the new batch query method for efficient bulk SELECT
            match repo.batch_find_by_paths_batch(chunk).await {
                Ok(existing_files) => {
                    // Create a HashMap for O(1) lookup
                    use std::collections::HashMap;
                    let existing_map: HashMap<String, &MediaFile> = existing_files
                        .iter()
                        .map(|f| (f.file_path.clone(), f))
                        .collect();

                    for path in chunk {
                        let path_str = path.to_string_lossy().to_string();
                        match existing_map.get(&path_str) {
                            Some(existing) => {
                                // File exists - check if modify_time changed
                                if let Ok(fs_metadata) = path.metadata() {
                                    if let Ok(fs_modify_time) = fs_metadata.modified() {
                                        let fs_time = fs_modify_time
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs();

                                        let db_time = existing.modify_time
                                            .map(|t| t.and_utc().timestamp() as u64)
                                            .unwrap_or(0);

                                        if fs_time == db_time {
                                            // Modify time unchanged - skip processing
                                            skip_list.push(path.clone());
                                        } else {
                                            // Modify time changed - needs update
                                            to_update += 1;
                                        }
                                    } else {
                                        // Failed to get fs modify time - treat as update
                                        to_update += 1;
                                    }
                                } else {
                                    // Failed to get metadata - treat as update
                                    to_update += 1;
                                }
                            }
                            None => {
                                // New file - needs processing
                                to_add += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Batch check failed: {}", e);
                    // Assume all files need to be added on error
                    to_add += chunk.len() as u64;
                }
            }
        }

        (to_add, to_update, skip_list)
    }

    /// Parallel metadata extraction using semaphore-controlled concurrency
    /// Reports results via scan_state for ordered progress updates
    async fn parallel_extract_metadata(&self, files: &[PathBuf]) -> Vec<ProcessingResult> {
        let concurrency = self.get_concurrency();
        let semaphore = Arc::new(Semaphore::new(concurrency));

        // Clone files to owned Vec for 'static lifetime
        let files_owned: Vec<PathBuf> = files.to_vec();
        let processors = self.processors.clone();
        let is_cancelled = self.is_cancelled.clone();
        let scan_state = self.scan_state.clone();

        // Use scoped spawn to avoid 'static lifetime requirement
        let mut handles = Vec::new();

        for path in &files_owned {
            let permit = semaphore.clone().acquire_owned();
            let path = path.clone();
            let processors = processors.clone();
            let is_cancelled = is_cancelled.clone();
            let scan_state = scan_state.clone();

            handles.push(tokio::spawn(async move {
                let _permit = permit.await;

                // Check if cancelled before processing
                if is_cancelled.load(Ordering::SeqCst) {
                    // Return None for cancelled files - they won't be counted
                    return None;
                }

                // Process the file
                match Self::extract_single_metadata(&path, &processors).await {
                    Ok(media_file) => {
                        scan_state.increment_success();
                        Some(ProcessingResult {
                            path,
                            success: Some(media_file),
                            error: None,
                        })
                    },
                    Err(e) => {
                        scan_state.increment_failure();
                        Some(ProcessingResult {
                            path,
                            success: None,
                            error: Some(e.to_string()),
                        })
                    },
                }
            }));
        }

        // Wait for all tasks to complete
        let mut all_results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(Some(result)) => all_results.push(result),
                // Cancelled tasks or panics are ignored (not counted as failures)
                _ => {}
            }
        }

        // Sort results to maintain order
        all_results.sort_by_key(|r| r.path.clone());

        all_results
    }

    /// Build a MediaFile from metadata extracted from a file.
    /// This function consolidates the MediaFile creation logic that was duplicated
    /// across extract_single_metadata, process_file_to_result, and process_file.
    fn build_media_file(
        path: &Path,
        file_name: String,
        file_type: &str,
        file_metadata: &MediaMetadata,
        format_metadata: &MediaMetadata,
    ) -> MediaFile {
        let mut media_file = MediaFile::new(
            path.to_string_lossy().to_string(),
            file_name,
            file_type.to_string(),
        );

        // Apply file metadata (file_size, create_time, modify_time)
        media_file.file_size = file_metadata.file_size;
        media_file.create_time = file_metadata.create_time;
        media_file.modify_time = file_metadata.modify_time;

        // Apply format-specific metadata
        media_file.mime_type = format_metadata.mime_type.clone();
        media_file.width = format_metadata.width;
        media_file.height = format_metadata.height;
        media_file.exif_timestamp = format_metadata.exif_timestamp;
        media_file.exif_timezone_offset = format_metadata.exif_timezone_offset.clone();
        media_file.camera_make = format_metadata.camera_make.clone();
        media_file.camera_model = format_metadata.camera_model.clone();
        media_file.lens_model = format_metadata.lens_model.clone();
        media_file.exposure_time = format_metadata.exposure_time.clone();
        media_file.aperture = format_metadata.aperture.clone();
        media_file.iso = format_metadata.iso;
        media_file.focal_length = format_metadata.focal_length.clone();
        media_file.duration = format_metadata.duration;
        media_file.video_codec = format_metadata.video_codec.clone();

        media_file
    }

    /// Extract metadata for a single file
    /// Uses spawn_blocking for synchronous file metadata extraction to avoid blocking async runtime
    async fn extract_single_metadata(
        path: &Path,
        processors: &ProcessorRegistry,
    ) -> Result<MediaFile, Box<dyn std::error::Error>> {
        let path_buf = path.to_path_buf();
        let processors = processors.clone();

        // Clone for spawn_blocking (since path_buf is moved into the closure)
        let path_for_blocking = path_buf.clone();
        // Run synchronous file metadata extraction in blocking thread pool
        let file_metadata = tokio::task::spawn_blocking(move || {
            crate::processors::file_metadata::extract_file_metadata(&path_for_blocking)
        }).await
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        // Extract format-specific metadata (async, may contain internal blocking operations)
        let processor = processors.find_processor(&path_buf).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Unsupported, "No processor found")
        })?;

        let format_metadata = processor.process(&path_buf).await?;

        // Build MediaFile using consolidated helper function
        let file_name = path_buf.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_type = if processor.media_type() == crate::processors::MediaType::Video {
            "video"
        } else {
            "image"
        };

        let media_file = Self::build_media_file(
            &path_buf,
            file_name,
            file_type,
            &file_metadata,
            &format_metadata,
        );

        Ok(media_file)
    }

    /// Batch write results to database and update last_scanned for unchanged files
    /// Returns true if the write was cancelled mid-way
    async fn batch_write_results_with_skip(
        &self,
        results: Vec<ProcessingResult>,
        skip_list: &[PathBuf],
        _total: u64
    ) -> bool {
        let batch_size = self.config.db_batch_write_size;
        let repo = MediaFileRepository::new(&self.db);

        let mut success_count = 0u64;
        let mut failure_count = 0u64;
        let mut cancelled = false;

        // Write processed files
        for chunk in results.chunks(batch_size) {
            // 检查是否需要取消，但先完成当前批次的处理
            let should_cancel = self.is_cancelled.load(Ordering::SeqCst);

            let files: Vec<MediaFile> = chunk.iter()
                .filter_map(|r| r.success.clone())
                .collect();

            if !files.is_empty() {
                match repo.batch_upsert(&files).await {
                    Ok(_) => {
                        success_count += files.len() as u64;
                    }
                    Err(e) => {
                        tracing::error!("Batch upsert failed: {}", e);
                        failure_count += files.len() as u64;
                    }
                }
            }

            for r in chunk {
                if r.success.is_none() {
                    failure_count += 1;
                    tracing::warn!("Failed to process {}: {}", r.path.display(), r.error.clone().unwrap_or_default());
                }
            }

            self.success_count.store(success_count, Ordering::SeqCst);
            self.failure_count.store(failure_count, Ordering::SeqCst);

            // 在完成当前批次后，如果检测到取消，则退出
            if should_cancel {
                cancelled = true;
                tracing::info!("Scan cancelled during writing, saved {} files so far", success_count);
                break;
            }
        }

        // Update last_scanned for unchanged files (batch touch)
        // Even if cancelled, we still update skip_list for files that weren't processed
        if !skip_list.is_empty() && !cancelled {
            if let Err(e) = repo.batch_touch(skip_list).await {
                tracing::error!("Batch touch failed: {}", e);
            }
        }

        cancelled
    }

    /// Calculate changes (serial fallback - uses DB per file)
    async fn calculate_changes(&self, files: &[PathBuf]) -> ScanProgress {
        let repo = MediaFileRepository::new(&self.db);
        let mut to_add = 0;
        let mut to_update = 0;

        for path in files {
            match repo.find_by_path(path).await {
                Ok(Some(_)) => to_update += 1,
                Ok(None) => to_add += 1,
                Err(_) => to_add += 1,
            }
        }

        ScanProgress {
            files_to_add: to_add,
            files_to_update: to_update,
            files_to_delete: 0,
            ..Default::default()
        }
    }

    /// Process files serially (fallback mode)
    async fn process_serial(&self, files: &[PathBuf]) {
        let total = files.len() as u64;
        let mut results: Vec<ProcessingResult> = Vec::with_capacity(total as usize);

        for (_, file) in files.iter().enumerate() {
            if self.is_cancelled.load(Ordering::SeqCst) {
                // 保存已处理的文件后再发送取消状态
                self.save_partial_results(&results, files).await;
                self.scan_state.cancelled();
                return;
            }

            match self.process_file_to_result(file).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    tracing::error!("Failed to process {}: {}", file.display(), e);
                    self.failure_count.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    }

    /// Process single file and return ProcessingResult
    async fn process_file_to_result(&self, path: &Path) -> Result<ProcessingResult, Box<dyn std::error::Error>> {
        let processor = self.processors.find_processor(path).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Unsupported, "No processor found")
        })?;

        let file_metadata = crate::processors::file_metadata::extract_file_metadata(path);
        let format_metadata = processor.process(path).await?;

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_type = if processor.media_type() == crate::processors::MediaType::Video {
            "video"
        } else {
            "image"
        };

        let media_file = Self::build_media_file(
            path,
            file_name,
            file_type,
            &file_metadata,
            &format_metadata,
        );

        Ok(ProcessingResult {
            path: path.to_path_buf(),
            success: Some(media_file),
            error: None,
        })
    }

    /// Save partial results when scan is cancelled
    /// 保存已处理的文件到数据库，用于取消时保留已处理的数据
    async fn save_partial_results(&self, results: &[ProcessingResult], all_files: &[PathBuf]) {
        let repo = MediaFileRepository::new(&self.db);

        // 保存已处理成功的文件
        let success_files: Vec<MediaFile> = results.iter()
            .filter_map(|r| r.success.clone())
            .collect();

        if !success_files.is_empty() {
            match repo.batch_upsert(&success_files).await {
                Ok(_) => {
                    tracing::info!("Cancelled scan: saved {} processed files", success_files.len());
                }
                Err(e) => {
                    tracing::error!("Failed to upsert partial results on cancel: {}", e);
                }
            }
        }

        // 更新 skip_list 中文件的 last_scanned（未被处理的文件）
        use std::collections::HashSet;
        let processed_paths: HashSet<String> = results.iter()
            .filter_map(|r| r.success.as_ref().map(|f| f.file_path.clone()))
            .collect();

        let skip_list: Vec<PathBuf> = all_files.iter()
            .filter(|p| !processed_paths.contains(&p.to_string_lossy().to_string()))
            .cloned()
            .collect();

        if !skip_list.is_empty() {
            if let Err(e) = repo.batch_touch(&skip_list).await {
                tracing::error!("Failed to touch skip list on cancel: {}", e);
            }
        }
    }

    async fn delete_missing(&self, existing_files: &[PathBuf]) {
        // 检查是否已取消
        if self.is_cancelled.load(Ordering::SeqCst) {
            tracing::debug!("Skipping delete phase - scan was cancelled");
            return;
        }

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
