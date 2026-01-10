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
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.time.Instant;
import java.time.LocalDateTime;
import java.time.ZoneId;
import java.util.*;
import java.util.concurrent.*;
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

            // 发送扫描启动消息
            webSocketService.sendScanStarted();

            if (applicationConfig.isParallelScanEnabled()) {
                scanDirectoryParallel(rootPath.toFile());
            } else {
                scanDirectorySerial(rootPath.toFile());
            }

            logger.info("目录扫描完成");
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
    
    private void scanDirectorySerial(File rootPath) {
        Set<String> existingFilePaths = new HashSet<>();
        scanDirectoryRecursive(rootPath, null, existingFilePaths);
        detectAndDeleteMissingFiles(existingFilePaths);
    }
    
    private void scanDirectoryParallel(File rootPath) {
        logger.info("使用并行模式扫描目录");
        
        Set<String> existingFilePaths = ConcurrentHashMap.newKeySet();
        List<CompletableFuture<Void>> fileProcessingFutures = new ArrayList<>();
        List<MediaFile> mediaFilesToSave = Collections.synchronizedList(new ArrayList<>());
        
        scanDirectoryRecursiveParallel(rootPath, null, existingFilePaths, fileProcessingFutures, mediaFilesToSave);
        
        logger.info("等待所有文件处理完成，共 {} 个任务", fileProcessingFutures.size());
        
        try {
            CompletableFuture.allOf(fileProcessingFutures.toArray(new CompletableFuture[0])).get(30, TimeUnit.MINUTES);
            
            logger.info("批量保存媒体文件，共 {} 条记录", mediaFilesToSave.size());
            batchSaveMediaFiles(mediaFilesToSave);
            
            detectAndDeleteMissingFiles(existingFilePaths);
            
        } catch (TimeoutException e) {
            logger.error("扫描超时，部分文件可能未处理完成");
        } catch (Exception e) {
            logger.error("并行扫描过程中发生错误", e);
        }
        
        logger.info("并行扫描完成 - 总文件数: {}, 成功: {}, 失败: {}", 
            currentProgress.getTotalFiles(), currentProgress.getSuccessCount(), currentProgress.getFailureCount());
    }
    
    private void scanDirectoryRecursive(File directory, Directory parentDir, Set<String> existingFilePaths) {
        if (scanCancelled) {
            return;
        }

        if (!directory.isDirectory()) {
            return;
        }

        Directory dirEntity = createOrUpdateDirectory(directory, parentDir);

        File[] files = directory.listFiles();
        if (files == null) {
            return;
        }

        for (File file : files) {
            if (scanCancelled) {
                return;
            }

            if (file.isDirectory()) {
                scanDirectoryRecursive(file, dirEntity, existingFilePaths);
            } else if (file.isFile()) {
                String extension = getFileExtension(file.getName()).toLowerCase();
                if (!isSupportedExtension(extension)) {
                    continue; // 跳过不支持的文件类型
                }
                existingFilePaths.add(file.getAbsolutePath());
                processFile(file, dirEntity);
            }
        }
    }

    private void scanDirectoryRecursiveParallel(File directory, Directory parentDir,
                                                  Set<String> existingFilePaths,
                                                  List<CompletableFuture<Void>> fileProcessingFutures,
                                                  List<MediaFile> mediaFilesToSave) {
        if (scanCancelled) {
            return;
        }

        if (!directory.isDirectory()) {
            return;
        }

        Directory dirEntity = createOrUpdateDirectory(directory, parentDir);

        File[] files = directory.listFiles();
        if (files == null) {
            return;
        }

        for (File file : files) {
            if (scanCancelled) {
                return;
            }

            if (file.isDirectory()) {
                scanDirectoryRecursiveParallel(file, dirEntity, existingFilePaths, fileProcessingFutures, mediaFilesToSave);
            } else if (file.isFile()) {
                String extension = getFileExtension(file.getName()).toLowerCase();
                if (!isSupportedExtension(extension)) {
                    continue; // 跳过不支持的文件类型
                }
                existingFilePaths.add(file.getAbsolutePath());
                CompletableFuture<Void> future = processFileAsync(file, dirEntity, mediaFilesToSave);
                fileProcessingFutures.add(future);
            }
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
    
    @Async("fileScanExecutor")
    private CompletableFuture<Void> processFileAsync(File file, Directory directory, List<MediaFile> mediaFilesToSave) {
        return CompletableFuture.runAsync(() -> {
            try {
                if (scanCancelled) {
                    return;
                }

                currentProgress.incrementTotalFiles();
                String fileName = file.getName();
                String extension = getFileExtension(fileName).toLowerCase();

                MediaFile mediaFile = null;
                if (IMAGE_EXTENSIONS.contains(extension)) {
                    mediaFile = processImageFile(file, directory, true);
                } else if (VIDEO_EXTENSIONS.contains(extension)) {
                    mediaFile = processVideoFile(file, directory, true);
                }

                if (mediaFile != null) {
                    mediaFilesToSave.add(mediaFile);
                    currentProgress.incrementSuccessCount();
                } else {
                    currentProgress.incrementFailureCount();
                    logger.warn("文件处理失败: {}, 扩展名: {}", file.getAbsolutePath(), extension);
                }

                // 每处理一定数量的文件后推送进度更新
                long processedCount = currentProgress.getSuccessCount() + currentProgress.getFailureCount();
                if (processedCount - lastProgressUpdateCount >= PROGRESS_UPDATE_INTERVAL) {
                    lastProgressUpdateCount = processedCount;
                    webSocketService.sendProgressUpdate(
                        currentProgress.getTotalFiles(),
                        currentProgress.getSuccessCount(),
                        currentProgress.getFailureCount(),
                        String.format("%.2f", currentProgress.getProgressPercentage())
                    );
                }
            } catch (Exception e) {
                logger.error("异步处理文件失败: {}", file.getName(), e);
                currentProgress.incrementFailureCount();
            }
        });
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
    
    public static class ScanProgress {
        private final AtomicLong totalFiles = new AtomicLong(0);
        private final AtomicLong successCount = new AtomicLong(0);
        private final AtomicLong failureCount = new AtomicLong(0);
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
    }
}
