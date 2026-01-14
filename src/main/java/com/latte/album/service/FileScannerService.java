package com.latte.album.service;

import com.latte.album.config.ApplicationConfig;
import com.latte.album.model.Directory;
import com.latte.album.model.MediaFile;
import com.latte.album.repository.DirectoryRepository;
import com.latte.album.repository.MediaFileRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import jakarta.annotation.PostConstruct;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.time.Instant;
import java.time.LocalDateTime;
import java.time.ZoneId;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import java.util.stream.Collectors;

@Service
public class FileScannerService {

    private static final Logger logger = LoggerFactory.getLogger(FileScannerService.class);
    private static final long PROGRESS_UPDATE_INTERVAL = 10; // 每处理10个文件更新一次进度

    private final ApplicationConfig applicationConfig;
    private final MediaFileRepository mediaFileRepository;
    private final DirectoryRepository directoryRepository;
    private final MediaProcessorService mediaProcessorService;
    private final ScanProgressWebSocketService webSocketService;
    private final CacheService cacheService;

    private volatile boolean scanCancelled = false;
    private volatile boolean isScanning = false;
    private ScanProgress currentProgress;
    private long lastProgressUpdateCount = 0;
    
    private static final Set<String> IMAGE_EXTENSIONS = new HashSet<>(Arrays.asList(
        ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp", ".tiff", ".heif", ".heic"
    ));
    
    private static final Set<String> VIDEO_EXTENSIONS = new HashSet<>(Arrays.asList(
        ".mp4", ".avi", ".mov", ".mkv", ".wmv", ".flv", ".webm"
    ));
    
    @Autowired
    public FileScannerService(ApplicationConfig applicationConfig,
                              MediaFileRepository mediaFileRepository,
                              DirectoryRepository directoryRepository,
                              MediaProcessorService mediaProcessorService,
                              ScanProgressWebSocketService webSocketService,
                              CacheService cacheService) {
        this.applicationConfig = applicationConfig;
        this.mediaFileRepository = mediaFileRepository;
        this.directoryRepository = directoryRepository;
        this.mediaProcessorService = mediaProcessorService;
        this.webSocketService = webSocketService;
        this.cacheService = cacheService;
    }
    
    @PostConstruct
    public void init() {
        logger.info("检查是否需要执行首次启动全量扫描");
        
        try {
            long mediaFileCount = mediaFileRepository.count();
            
            if (mediaFileCount == 0) {
                logger.info("检测到首次启动（数据库中无媒体文件记录），开始执行全量文件扫描");
                scanDirectory();
            } else {
                logger.info("数据库中已存在 {} 条媒体文件记录，跳过首次启动扫描", mediaFileCount);
            }
        } catch (Exception e) {
            logger.error("首次启动扫描检查失败", e);
        }
    }

    /**
     * 异步扫描目录 - 供 API 调用，避免阻塞
     */
    @Async("fileScanExecutor")
    public CompletableFuture<Void> scanDirectoryAsync() {
        scanDirectory();
        return CompletableFuture.completedFuture(null);
    }

    public void scanDirectory() {
        String basePath = applicationConfig.getBasePath();
        logger.info("开始扫描目录: {}", basePath);

        try {
            Path rootPath = Paths.get(basePath);
            if (!Files.exists(rootPath)) {
                logger.error("指定的目录不存在: {}", basePath);
                webSocketService.sendScanError("目录不存在: " + basePath);
                return;
            }

            isScanning = true;
            scanCancelled = false;
            currentProgress = new ScanProgress();
            lastProgressUpdateCount = 0;

            // 阶段1：收集所有文件列表
            logger.info("阶段1：收集文件列表...");
            webSocketService.sendPhaseUpdate("collecting", "正在收集文件列表...", 0, 0, 0, "0.00", currentProgress.getStartTime());
            Set<String> existingFilePaths = collectAllFiles(rootPath);

            // 阶段2：计算新增、修改、删除的文件数量
            logger.info("阶段2：计算变更数量...");
            webSocketService.sendPhaseUpdate("counting", "正在计算变更数量...", 0, 0, 0, "0.00", currentProgress.getStartTime());
            ScanCounts counts = calculateScanCounts(existingFilePaths);

            // 在 ScanProgress 中保存各阶段数量
            currentProgress.setFilesToAdd(counts.getFilesToAdd());
            currentProgress.setFilesToUpdate(counts.getFilesToUpdate());
            currentProgress.setFilesToDelete(counts.getFilesToDelete());

            logger.info("扫描统计 - 新增: {}, 修改: {}, 删除: {}",
                counts.getFilesToAdd(), counts.getFilesToUpdate(), counts.getFilesToDelete());

            // 发送扫描开始消息（包含各阶段数量）
            webSocketService.sendScanStarted(
                counts.getFilesToAdd(),
                counts.getFilesToUpdate(),
                counts.getFilesToDelete()
            );

            // 阶段3：处理新增和修改的文件
            if (counts.getFilesToAdd() > 0 || counts.getFilesToUpdate() > 0) {
                logger.info("阶段3：处理文件，共 {} 个文件需要处理", counts.getFilesToAdd() + counts.getFilesToUpdate());
                webSocketService.sendPhaseUpdate("processing", "正在处理文件...",
                    counts.getFilesToAdd() + counts.getFilesToUpdate(), 0, 0, "0.00", currentProgress.getStartTime(),
                    counts.getFilesToAdd(), counts.getFilesToUpdate(), counts.getFilesToDelete());

                if (applicationConfig.isParallelScanEnabled()) {
                    processFilesParallel(existingFilePaths, counts);
                } else {
                    processFilesSerial(existingFilePaths, counts);
                }
            }

            // 阶段4：删除不存在的文件
            if (counts.getFilesToDelete() > 0) {
                logger.info("阶段4：清理不存在的文件，共 {} 个文件需要删除", counts.getFilesToDelete());
                webSocketService.sendPhaseUpdate("deleting", "正在清理不存在的文件...",
                    currentProgress.getSuccessCount(), currentProgress.getSuccessCount(), currentProgress.getFailureCount(),
                    String.format("%.2f", currentProgress.getProgressPercentage()), currentProgress.getStartTime(),
                    counts.getFilesToAdd(), counts.getFilesToUpdate(), counts.getFilesToDelete());
                detectAndDeleteMissingFiles(existingFilePaths);
            }

            logger.info("目录扫描完成 - 总文件: {}, 成功: {}, 失败: {}",
                currentProgress.getTotalFiles(), currentProgress.getSuccessCount(), currentProgress.getFailureCount());
        } catch (Exception e) {
            logger.error("扫描过程中发生错误", e);
            webSocketService.sendScanError(e.getMessage());
        } finally {
            isScanning = false;
            // 发送扫描完成消息
            webSocketService.sendScanCompleted(
                currentProgress.getTotalFiles(),
                currentProgress.getSuccessCount(),
                currentProgress.getFailureCount()
            );
        }
    }
    
    private boolean isSupportedExtension(String extension) {
        return IMAGE_EXTENSIONS.contains(extension) || VIDEO_EXTENSIONS.contains(extension);
    }

    private Directory createOrUpdateDirectory(File directory, Directory parentDir) {
        String path = directory.getAbsolutePath();
        return directoryRepository.findByPath(path).orElseGet(() -> {
            Directory newDir = new Directory();
            newDir.setPath(path);
            newDir.setParentId(parentDir != null ? parentDir.getId() : null);
            newDir.setLastModified(LocalDateTime.now());
            return directoryRepository.save(newDir);
        });
    }
    
    private void processFile(File file, Directory directory) {
        String fileName = file.getName();
        String extension = getFileExtension(fileName).toLowerCase();
        
        if (IMAGE_EXTENSIONS.contains(extension)) {
            processImageFile(file, directory);
        } else if (VIDEO_EXTENSIONS.contains(extension)) {
            processVideoFile(file, directory);
        }
    }
    
    private void processImageFile(File file, Directory directory) {
        processImageFile(file, directory, false);
    }
    
    private MediaFile processImageFile(File file, Directory directory, boolean async) {
        String filePath = file.getAbsolutePath();
        MediaFile mediaFile = mediaFileRepository.findByFilePath(filePath).orElseGet(() -> {
            MediaFile newFile = new MediaFile();
            newFile.setFilePath(filePath);
            newFile.setFileName(file.getName());
            newFile.setFileType("image");
            newFile.setFileSize(file.length());
            newFile.setCreateTime(LocalDateTime.now());
            newFile.setModifyTime(LocalDateTime.now());
            return newFile;
        });
        
        LocalDateTime lastModified = LocalDateTime.ofInstant(
            Instant.ofEpochMilli(file.lastModified()), ZoneId.systemDefault());
        
        if (mediaFile.getLastScanned() == null || 
            mediaFile.getLastScanned().isBefore(lastModified)) {
            
            mediaFile.setModifyTime(lastModified);
            mediaFile.setLastScanned(LocalDateTime.now());
            
            mediaProcessorService.processImage(file, mediaFile);
            
            if (!async) {
                mediaFileRepository.save(mediaFile);
            }
            
            logger.debug("处理图片文件: {}", filePath);
        }
        
        return mediaFile;
    }
    
    private void processVideoFile(File file, Directory directory) {
        processVideoFile(file, directory, false);
    }
    
    private MediaFile processVideoFile(File file, Directory directory, boolean async) {
        String filePath = file.getAbsolutePath();
        MediaFile mediaFile = mediaFileRepository.findByFilePath(filePath).orElseGet(() -> {
            MediaFile newFile = new MediaFile();
            newFile.setFilePath(filePath);
            newFile.setFileName(file.getName());
            newFile.setFileType("video");
            newFile.setFileSize(file.length());
            newFile.setCreateTime(LocalDateTime.now());
            newFile.setModifyTime(LocalDateTime.now());
            return newFile;
        });
        
        LocalDateTime lastModified = LocalDateTime.ofInstant(
            Instant.ofEpochMilli(file.lastModified()), ZoneId.systemDefault());
        
        if (mediaFile.getLastScanned() == null || 
            mediaFile.getLastScanned().isBefore(lastModified)) {
            
            mediaFile.setModifyTime(lastModified);
            mediaFile.setLastScanned(LocalDateTime.now());
            
            mediaProcessorService.processVideo(file, mediaFile);
            
            if (!async) {
                mediaFileRepository.save(mediaFile);
            }
            
            logger.debug("处理视频文件: {}", filePath);
        }
        
        return mediaFile;
    }
    
    @Transactional
    private void batchSaveMediaFiles(List<MediaFile> mediaFiles) {
        if (mediaFiles.isEmpty()) {
            return;
        }
        
        int batchSize = applicationConfig.getParallelBatchSize();
        int total = mediaFiles.size();
        
        for (int i = 0; i < total; i += batchSize) {
            int end = Math.min(i + batchSize, total);
            List<MediaFile> batch = mediaFiles.subList(i, end);
            mediaFileRepository.saveAll(batch);
            logger.debug("批量保存媒体文件: {}/{}", end, total);
        }
    }
    
    private void detectAndDeleteMissingFiles(Set<String> existingFilePaths) {
        logger.info("开始检测文件系统中已不存在的数据库记录");

        try {
            List<MediaFile> allMediaFiles = mediaFileRepository.findAll();
            int deletedCount = 0;

            for (MediaFile mediaFile : allMediaFiles) {
                String filePath = mediaFile.getFilePath();

                if (!existingFilePaths.contains(filePath)) {
                    // 先删除缩略图缓存，再删除数据库记录
                    String fileId = mediaFile.getId();
                    cacheService.deleteThumbnails(fileId);
                    mediaFileRepository.delete(mediaFile);
                    deletedCount++;
                    logger.info("删除数据库中已不存在的文件记录: {}", filePath);
                }
            }

            if (deletedCount > 0) {
                logger.info("共删除 {} 条已不存在的文件记录", deletedCount);
            } else {
                logger.info("未发现需要删除的文件记录");
            }
        } catch (Exception e) {
            logger.error("检测和删除缺失文件时发生错误", e);
        }
    }
    
    private String getFileExtension(String fileName) {
        int lastDotIndex = fileName.lastIndexOf('.');
        if (lastDotIndex > 0) {
            return fileName.substring(lastDotIndex);
        }
        return "";
    }
    
    public boolean isScanning() {
        return isScanning;
    }
    
    public ScanProgress getScanProgress() {
        return currentProgress;
    }
    
    public void cancelScan() {
        scanCancelled = true;
        logger.info("已请求取消扫描");
        webSocketService.sendScanCancelled();
    }
    
    /**
     * 扫描计数统计类
     */
    public static class ScanCounts {
        private long filesToAdd = 0;
        private long filesToUpdate = 0;
        private long filesToDelete = 0;

        public long getFilesToAdd() {
            return filesToAdd;
        }

        public void setFilesToAdd(long filesToAdd) {
            this.filesToAdd = filesToAdd;
        }

        public long getFilesToUpdate() {
            return filesToUpdate;
        }

        public void setFilesToUpdate(long filesToUpdate) {
            this.filesToUpdate = filesToUpdate;
        }

        public long getFilesToDelete() {
            return filesToDelete;
        }

        public void setFilesToDelete(long filesToDelete) {
            this.filesToDelete = filesToDelete;
        }

        public long getTotalToProcess() {
            return filesToAdd + filesToUpdate;
        }
    }

    public static class ScanProgress {
        private final AtomicLong totalFiles = new AtomicLong(0);
        private final AtomicLong successCount = new AtomicLong(0);
        private final AtomicLong failureCount = new AtomicLong(0);
        private final AtomicLong filesToDelete = new AtomicLong(0);
        private final AtomicLong filesToAdd = new AtomicLong(0);
        private final AtomicLong filesToUpdate = new AtomicLong(0);
        private final LocalDateTime startTime = LocalDateTime.now();
        
        public long getTotalFiles() {
            return totalFiles.get();
        }
        
        public long getSuccessCount() {
            return successCount.get();
        }
        
        public long getFailureCount() {
            return failureCount.get();
        }
        
        public LocalDateTime getStartTime() {
            return startTime;
        }
        
        public void incrementTotalFiles() {
            totalFiles.incrementAndGet();
        }
        
        public void incrementSuccessCount() {
            successCount.incrementAndGet();
        }
        
        public void incrementFailureCount() {
            failureCount.incrementAndGet();
        }
        
        public double getProgressPercentage() {
            long total = getTotalFiles();
            if (total == 0) {
                return 0.0;
            }
            return (getSuccessCount() + getFailureCount()) * 100.0 / total;
        }

        public long getFilesToDelete() {
            return filesToDelete.get();
        }

        public void setFilesToDelete(long count) {
            filesToDelete.set(count);
        }

        public void setTotalFiles(long count) {
            totalFiles.set(count);
        }

        public void setSuccessCount(long count) {
            successCount.set(count);
        }

        public void setFailureCount(long count) {
            failureCount.set(count);
        }

        public long getFilesToAdd() {
            return filesToAdd.get();
        }

        public void setFilesToAdd(long count) {
            filesToAdd.set(count);
        }

        public long getFilesToUpdate() {
            return filesToUpdate.get();
        }

        public void setFilesToUpdate(long count) {
            filesToUpdate.set(count);
        }
    }

    /**
     * 阶段1：收集所有文件列表
     */
    private Set<String> collectAllFiles(Path rootPath) {
        Set<String> filePaths = new HashSet<>();
        collectFilesRecursive(rootPath, filePaths);
        logger.info("共收集到 {} 个支持的文件", filePaths.size());
        return filePaths;
    }

    /**
     * 递归收集文件路径
     */
    private void collectFilesRecursive(Path directory, Set<String> filePaths) {
        if (scanCancelled) {
            return;
        }

        try {
            try (var stream = Files.walk(directory)) {
                stream.filter(Files::isRegularFile)
                    .filter(path -> {
                        String extension = getFileExtension(path.getFileName().toString()).toLowerCase();
                        return isSupportedExtension(extension);
                    })
                    .map(Path::toAbsolutePath)
                    .map(Path::toString)
                    .forEach(filePaths::add);
            }
        } catch (IOException e) {
            logger.error("遍历目录失败: {}", directory, e);
        }
    }

    /**
     * 阶段2：计算新增、修改、删除的文件数量
     */
    private ScanCounts calculateScanCounts(Set<String> existingFilePaths) {
        ScanCounts counts = new ScanCounts();
        List<MediaFile> allMediaFiles = mediaFileRepository.findAll();

        for (MediaFile mediaFile : allMediaFiles) {
            String filePath = mediaFile.getFilePath();

            if (!existingFilePaths.contains(filePath)) {
                // 文件已删除
                counts.setFilesToDelete(counts.getFilesToDelete() + 1);
            } else {
                // 文件存在，检查是否需要更新
                Path path = Paths.get(filePath);
                if (Files.exists(path)) {
                    LocalDateTime lastModified = LocalDateTime.ofInstant(
                        Instant.ofEpochMilli(path.toFile().lastModified()), ZoneId.systemDefault());

                    if (mediaFile.getLastScanned() == null ||
                        mediaFile.getLastScanned().isBefore(lastModified)) {
                        // 文件已修改
                        counts.setFilesToUpdate(counts.getFilesToUpdate() + 1);
                    }
                }
                // 从existingFilePaths中移除已处理的文件路径
                existingFilePaths.remove(filePath);
            }
        }

        // 剩余的paths就是要新增的文件
        counts.setFilesToAdd(existingFilePaths.size());

        return counts;
    }

    /**
     * 阶段3：串行处理文件
     */
    private void processFilesSerial(Set<String> filePaths, ScanCounts counts) {
        int processed = 0;
        int total = (int) counts.getTotalToProcess();

        for (String filePath : filePaths) {
            if (scanCancelled) {
                return;
            }

            // 检查是新增还是修改
            boolean isNew = counts.getFilesToAdd() > 0 && processed < counts.getFilesToAdd();
            if (!isNew && counts.getFilesToUpdate() > 0) {
                // 检查是否在修改列表中
                Path path = Paths.get(filePath);
                if (Files.exists(path)) {
                    LocalDateTime lastModified = LocalDateTime.ofInstant(
                        Instant.ofEpochMilli(path.toFile().lastModified()), ZoneId.systemDefault());

                    MediaFile mediaFile = mediaFileRepository.findByFilePath(filePath).orElse(null);
                    if (mediaFile != null && mediaFile.getLastScanned() != null &&
                        !mediaFile.getLastScanned().isBefore(lastModified)) {
                        continue; // 不需要更新
                    }
                }
            }

            File file = new File(filePath);
            processFile(file, null);
            processed++;

            // 更新进度
            currentProgress.incrementTotalFiles();
            currentProgress.incrementSuccessCount();

            if (processed % PROGRESS_UPDATE_INTERVAL == 0) {
                webSocketService.sendPhaseUpdate("processing", "正在处理文件...",
                    total, processed, 0,
                    String.format("%.2f", processed * 100.0 / total), currentProgress.getStartTime(),
                    counts.getFilesToAdd(), counts.getFilesToUpdate(), counts.getFilesToDelete());
            }
        }
    }

    /**
     * 阶段3：并行处理文件
     */
    private void processFilesParallel(Set<String> filePaths, ScanCounts counts) {
        Set<String> concurrentFilePaths = ConcurrentHashMap.newKeySet();
        concurrentFilePaths.addAll(filePaths);

        List<CompletableFuture<Void>> fileProcessingFutures = new ArrayList<>();
        List<MediaFile> mediaFilesToSave = Collections.synchronizedList(new ArrayList<>());
        AtomicInteger processedCount = new AtomicInteger(0);
        AtomicInteger failedCount = new AtomicInteger(0);
        int total = (int) counts.getTotalToProcess();

        for (String filePath : concurrentFilePaths) {
            if (scanCancelled) {
                return;
            }

            CompletableFuture<Void> future = processFileAsync(filePath, mediaFilesToSave, processedCount, failedCount);
            fileProcessingFutures.add(future);
        }

        logger.info("等待所有文件处理完成，共 {} 个任务", fileProcessingFutures.size());

        // 定期发送进度更新
        ScheduledExecutorService scheduler = Executors.newSingleThreadScheduledExecutor();
        scheduler.scheduleAtFixedRate(() -> {
            if (scanCancelled || !isScanning) {
                scheduler.shutdown();
                return;
            }

            int processed = processedCount.get() + failedCount.get();
            if (processed > 0 && processed <= total) {
                webSocketService.sendPhaseUpdate("processing", "正在处理文件...",
                    total, processedCount.get(), failedCount.get(),
                    String.format("%.2f", processed * 100.0 / total), currentProgress.getStartTime(),
                    counts.getFilesToAdd(), counts.getFilesToUpdate(), counts.getFilesToDelete());
            }
        }, 500, 500, TimeUnit.MILLISECONDS);

        try {
            CompletableFuture.allOf(fileProcessingFutures.toArray(new CompletableFuture[0])).get(30, TimeUnit.MINUTES);

            scheduler.shutdown();
            logger.info("批量保存媒体文件，共 {} 条记录", mediaFilesToSave.size());
            batchSaveMediaFiles(mediaFilesToSave);

            currentProgress.setTotalFiles(total);
            currentProgress.setSuccessCount(mediaFilesToSave.size());

            // 发送完成进度
            webSocketService.sendPhaseUpdate("processing", "文件处理完成",
                total, mediaFilesToSave.size(), 0,
                String.format("%.2f", 100.0), currentProgress.getStartTime(),
                counts.getFilesToAdd(), counts.getFilesToUpdate(), counts.getFilesToDelete());

        } catch (TimeoutException e) {
            logger.error("扫描超时，部分文件可能未处理完成");
            scheduler.shutdown();
        } catch (Exception e) {
            logger.error("并行扫描过程中发生错误", e);
            scheduler.shutdown();
        }
    }

    /**
     * 异步处理单个文件
     */
    @Async("fileScanExecutor")
    private CompletableFuture<Void> processFileAsync(String filePath, List<MediaFile> mediaFilesToSave,
                                                      AtomicInteger processedCount, AtomicInteger failedCount) {
        return CompletableFuture.runAsync(() -> {
            try {
                if (scanCancelled) {
                    return;
                }

                File file = new File(filePath);
                if (!file.exists() || !file.isFile()) {
                    return;
                }

                String extension = getFileExtension(file.getName()).toLowerCase();
                MediaFile mediaFile = null;

                if (IMAGE_EXTENSIONS.contains(extension)) {
                    mediaFile = processImageFile(file, null, true);
                } else if (VIDEO_EXTENSIONS.contains(extension)) {
                    mediaFile = processVideoFile(file, null, true);
                }

                if (mediaFile != null) {
                    synchronized (mediaFilesToSave) {
                        mediaFilesToSave.add(mediaFile);
                    }
                    processedCount.incrementAndGet();
                } else {
                    failedCount.incrementAndGet();
                    logger.warn("文件处理失败: {}, 扩展名: {}", file.getAbsolutePath(), extension);
                }
            } catch (Exception e) {
                failedCount.incrementAndGet();
                logger.error("异步处理文件失败: {}", filePath, e);
            }
        });
    }
}
