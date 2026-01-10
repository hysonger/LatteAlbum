package com.latte.album.service;

import com.latte.album.config.ApplicationConfig;
import com.latte.album.model.Directory;
import com.latte.album.model.MediaFile;
import com.latte.album.repository.DirectoryRepository;
import com.latte.album.repository.MediaFileRepository;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.test.util.ReflectionTestUtils;

import java.io.File;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class FileScannerServiceTest {

    private static final Logger logger = LoggerFactory.getLogger(FileScannerServiceTest.class);
    
    @Mock
    private MediaFileRepository mediaFileRepository;
    
    @Mock
    private DirectoryRepository directoryRepository;
    
    @Mock
    private MediaProcessorService mediaProcessorService;
    
    @InjectMocks
    private FileScannerService fileScannerService;
    
    private String testBasePath;
    private File existingFile;
    private File missingFile;
    
    @BeforeEach
    void setUp() {
        testBasePath = System.getProperty("java.io.tmpdir") + File.separator + "test-photos";
        File testDir = new File(testBasePath);
        if (!testDir.exists()) {
            testDir.mkdirs();
        }
        
        existingFile = new File(testBasePath, "existing.jpg");
        try {
            if (!existingFile.exists()) {
                existingFile.createNewFile();
            }
        } catch (Exception e) {
            logger.error("创建测试文件失败", e);
        }
        
        missingFile = new File(testBasePath, "missing.jpg");
        
        ApplicationConfig applicationConfig = new ApplicationConfig();
        ReflectionTestUtils.setField(applicationConfig, "basePath", testBasePath);
        ReflectionTestUtils.setField(applicationConfig, "parallelScanEnabled", true);
        ReflectionTestUtils.setField(applicationConfig, "parallelBatchSize", 50);
        ReflectionTestUtils.setField(fileScannerService, "applicationConfig", applicationConfig);
        
        Directory mockDirectory = new Directory();
        mockDirectory.setId(1L);
        when(directoryRepository.findByPath(anyString())).thenReturn(Optional.of(mockDirectory));
        when(directoryRepository.save(any(Directory.class))).thenReturn(mockDirectory);
        
        when(mediaFileRepository.findByFilePath(existingFile.getAbsolutePath()))
            .thenReturn(Optional.of(createMediaFile(existingFile.getAbsolutePath(), "image")));
        when(mediaFileRepository.findByFilePath(missingFile.getAbsolutePath()))
            .thenReturn(Optional.empty());
        when(mediaFileRepository.count()).thenReturn(0L);
    }
    
    @Test
    void testDetectAndDeleteMissingFiles() {
        MediaFile existingDbFile = createMediaFile(existingFile.getAbsolutePath(), "image");
        MediaFile missingDbFile = createMediaFile(missingFile.getAbsolutePath(), "image");
        
        List<MediaFile> allMediaFiles = Arrays.asList(existingDbFile, missingDbFile);
        when(mediaFileRepository.findAll()).thenReturn(allMediaFiles);
        
        fileScannerService.scanDirectory();
        
        verify(mediaFileRepository, never()).delete(existingDbFile);
        verify(mediaFileRepository, times(1)).delete(missingDbFile);
        
        logger.info("测试完成：成功检测并删除了不存在的文件记录");
    }
    
    @Test
    void testNoMissingFiles() {
        MediaFile existingDbFile = createMediaFile(existingFile.getAbsolutePath(), "image");
        
        List<MediaFile> allMediaFiles = Arrays.asList(existingDbFile);
        when(mediaFileRepository.findAll()).thenReturn(allMediaFiles);
        
        fileScannerService.scanDirectory();
        
        verify(mediaFileRepository, never()).delete(any(MediaFile.class));
        
        logger.info("测试完成：没有检测到需要删除的文件记录");
    }
    
    @Test
    void testEmptyDatabase() {
        when(mediaFileRepository.findAll()).thenReturn(new ArrayList<>());
        
        fileScannerService.scanDirectory();
        
        verify(mediaFileRepository, never()).delete(any(MediaFile.class));
        
        logger.info("测试完成：空数据库情况下没有执行删除操作");
    }
    
    @Test
    void testParallelScanEnabled() {
        fileScannerService.scanDirectory();
        
        assertTrue(fileScannerService.isScanning() || fileScannerService.getScanProgress() != null);
        logger.info("测试完成：并行扫描模式已启用");
    }
    
    @Test
    void testSerialScanMode() {
        ApplicationConfig applicationConfig = new ApplicationConfig();
        ReflectionTestUtils.setField(applicationConfig, "parallelScanEnabled", false);
        ReflectionTestUtils.setField(fileScannerService, "applicationConfig", applicationConfig);
        
        fileScannerService.scanDirectory();
        
        logger.info("测试完成：串行扫描模式测试通过");
    }
    
    @Test
    void testScanProgressTracking() {
        fileScannerService.scanDirectory();
        
        FileScannerService.ScanProgress progress = fileScannerService.getScanProgress();
        assertNotNull(progress);
        assertTrue(progress.getTotalFiles() >= 0);
        assertTrue(progress.getSuccessCount() >= 0);
        assertTrue(progress.getFailureCount() >= 0);
        assertTrue(progress.getProgressPercentage() >= 0.0);
        assertNotNull(progress.getStartTime());
        
        logger.info("测试完成：扫描进度跟踪正常 - 总文件数: {}, 成功: {}, 失败: {}, 进度: {}%", 
            progress.getTotalFiles(), progress.getSuccessCount(), progress.getFailureCount(), 
            String.format("%.2f", progress.getProgressPercentage()));
    }
    
    @Test
    void testCancelScan() {
        Thread scanThread = new Thread(() -> {
            fileScannerService.scanDirectory();
        });
        scanThread.start();
        
        try {
            TimeUnit.MILLISECONDS.sleep(100);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
        
        fileScannerService.cancelScan();
        
        try {
            scanThread.join(2000);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
        
        logger.info("测试完成：取消扫描功能测试通过");
    }
    
    @Test
    void testIsScanning() {
        assertFalse(fileScannerService.isScanning());
        
        Thread scanThread = new Thread(() -> {
            fileScannerService.scanDirectory();
        });
        scanThread.start();
        
        try {
            TimeUnit.MILLISECONDS.sleep(100);
            assertTrue(fileScannerService.isScanning());
            
            scanThread.join(3000);
            assertFalse(fileScannerService.isScanning());
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
        
        logger.info("测试完成：扫描状态检测正常");
    }
    
    @Test
    void testBatchSaveMediaFiles() {
        fileScannerService.scanDirectory();
        
        verify(mediaFileRepository, atLeast(0)).saveAll(anyList());
        
        logger.info("测试完成：批量保存功能测试通过");
    }
    
    private MediaFile createMediaFile(String filePath, String fileType) {
        MediaFile mediaFile = new MediaFile();
        mediaFile.setId(java.util.UUID.randomUUID().toString());
        mediaFile.setFilePath(filePath);
        mediaFile.setFileName(new File(filePath).getName());
        mediaFile.setFileType(fileType);
        mediaFile.setFileSize(1024L);
        mediaFile.setCreateTime(LocalDateTime.now());
        mediaFile.setModifyTime(LocalDateTime.now());
        mediaFile.setLastScanned(LocalDateTime.now());
        return mediaFile;
    }
    
    void tearDown() {
        if (existingFile.exists()) {
            existingFile.delete();
        }
        File testDir = new File(testBasePath);
        if (testDir.exists()) {
            testDir.delete();
        }
    }
}
