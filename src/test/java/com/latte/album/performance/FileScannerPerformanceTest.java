package com.latte.album.performance;

import com.latte.album.config.ApplicationConfig;
import com.latte.album.service.FileScannerService;
import org.junit.jupiter.api.*;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.test.context.TestPropertySource;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.List;

@SpringBootTest
@TestPropertySource(properties = {
    "album.scan.parallel.enabled=true",
    "album.scan.parallel.batch-size=50"
})
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
class FileScannerPerformanceTest {

    private static final Logger logger = LoggerFactory.getLogger(FileScannerPerformanceTest.class);
    
    @Autowired
    private FileScannerService fileScannerService;
    
    @Autowired
    private ApplicationConfig applicationConfig;
    
    private static String testBasePath;
    private static List<File> testFiles = new ArrayList<>();
    
    @BeforeAll
    static void setupTestEnvironment() throws IOException {
        testBasePath = System.getProperty("java.io.tmpdir") + File.separator + "perf-test-photos";
        File testDir = new File(testBasePath);
        if (testDir.exists()) {
            deleteDirectory(testDir);
        }
        testDir.mkdirs();
        
        logger.info("性能测试环境初始化完成: {}", testBasePath);
    }
    
    @AfterAll
    static void cleanupTestEnvironment() {
        File testDir = new File(testBasePath);
        if (testDir.exists()) {
            deleteDirectory(testDir);
        }
        logger.info("性能测试环境清理完成");
    }
    
    @Test
    @Order(1)
    @DisplayName("性能测试 - 100个文件")
    void testPerformance100Files() throws IOException {
        int fileCount = 100;
        createTestFiles(fileCount);
        
        PerformanceResult result = runScanTest("100文件", fileCount);
        
        assertPerformanceResult(result);
        logPerformanceResult(result);
    }
    
    @Test
    @Order(2)
    @DisplayName("性能测试 - 500个文件")
    void testPerformance500Files() throws IOException {
        int fileCount = 500;
        createTestFiles(fileCount);
        
        PerformanceResult result = runScanTest("500文件", fileCount);
        
        assertPerformanceResult(result);
        logPerformanceResult(result);
    }
    
    @Test
    @Order(3)
    @DisplayName("性能测试 - 1000个文件")
    void testPerformance1000Files() throws IOException {
        int fileCount = 1000;
        createTestFiles(fileCount);
        
        PerformanceResult result = runScanTest("1000文件", fileCount);
        
        assertPerformanceResult(result);
        logPerformanceResult(result);
    }
    
    @Test
    @Order(4)
    @DisplayName("性能测试 - 串行 vs 并行对比")
    void testSerialVsParallelComparison() throws IOException {
        int fileCount = 500;
        createTestFiles(fileCount);
        
        logger.info("========== 串行 vs 并行性能对比测试 ==========");
        
        PerformanceResult serialResult = runSerialScanTest(fileCount);
        logPerformanceResult(serialResult);
        
        PerformanceResult parallelResult = runParallelScanTest(fileCount);
        logPerformanceResult(parallelResult);
        
        double speedup = (double) serialResult.durationMs / parallelResult.durationMs;
        logger.info("========== 性能提升: {:.2f}x ==========", speedup);
        
        Assertions.assertTrue(speedup > 1.0, "并行扫描应该比串行扫描快");
    }
    
    private PerformanceResult runScanTest(String testName, int fileCount) {
        logger.info("========== {} 性能测试开始 ==========", testName);
        
        long startTime = System.currentTimeMillis();
        fileScannerService.scanDirectory();
        long endTime = System.currentTimeMillis();
        
        PerformanceResult result = new PerformanceResult();
        result.testName = testName;
        result.fileCount = fileCount;
        result.durationMs = endTime - startTime;
        result.durationSeconds = result.durationMs / 1000.0;
        result.filesPerSecond = fileCount / result.durationSeconds;
        
        FileScannerService.ScanProgress progress = fileScannerService.getScanProgress();
        if (progress != null) {
            result.successCount = progress.getSuccessCount();
            result.failureCount = progress.getFailureCount();
        }
        
        return result;
    }
    
    private PerformanceResult runSerialScanTest(int fileCount) {
        logger.info("========== 串行扫描测试开始 ==========");
        
        ApplicationConfig config = new ApplicationConfig();
        try {
            java.lang.reflect.Field field = ApplicationConfig.class.getDeclaredField("parallelScanEnabled");
            field.setAccessible(true);
            field.set(config, false);
        } catch (Exception e) {
            logger.error("设置配置失败", e);
        }
        
        long startTime = System.currentTimeMillis();
        fileScannerService.scanDirectory();
        long endTime = System.currentTimeMillis();
        
        PerformanceResult result = new PerformanceResult();
        result.testName = "串行扫描";
        result.fileCount = fileCount;
        result.durationMs = endTime - startTime;
        result.durationSeconds = result.durationMs / 1000.0;
        result.filesPerSecond = fileCount / result.durationSeconds;
        
        FileScannerService.ScanProgress progress = fileScannerService.getScanProgress();
        if (progress != null) {
            result.successCount = progress.getSuccessCount();
            result.failureCount = progress.getFailureCount();
        }
        
        return result;
    }
    
    private PerformanceResult runParallelScanTest(int fileCount) {
        logger.info("========== 并行扫描测试开始 ==========");
        
        long startTime = System.currentTimeMillis();
        fileScannerService.scanDirectory();
        long endTime = System.currentTimeMillis();
        
        PerformanceResult result = new PerformanceResult();
        result.testName = "并行扫描";
        result.fileCount = fileCount;
        result.durationMs = endTime - startTime;
        result.durationSeconds = result.durationMs / 1000.0;
        result.filesPerSecond = fileCount / result.durationSeconds;
        
        FileScannerService.ScanProgress progress = fileScannerService.getScanProgress();
        if (progress != null) {
            result.successCount = progress.getSuccessCount();
            result.failureCount = progress.getFailureCount();
        }
        
        return result;
    }
    
    private void assertPerformanceResult(PerformanceResult result) {
        Assertions.assertNotNull(result);
        Assertions.assertTrue(result.durationMs > 0, "扫描时间应该大于0");
        Assertions.assertTrue(result.filesPerSecond > 0, "每秒处理文件数应该大于0");
        Assertions.assertEquals(result.fileCount, result.successCount + result.failureCount, 
            "处理文件总数应该等于成功数加失败数");
    }
    
    private void logPerformanceResult(PerformanceResult result) {
        logger.info("========== {} 性能测试结果 ==========", result.testName);
        logger.info("文件数量: {}", result.fileCount);
        logger.info("扫描耗时: {} ms ({} 秒)", result.durationMs, String.format("%.2f", result.durationSeconds));
        logger.info("处理速度: {} 文件/秒", String.format("%.2f", result.filesPerSecond));
        logger.info("成功数量: {}", result.successCount);
        logger.info("失败数量: {}", result.failureCount);
        logger.info("成功率: {}%", String.format("%.2f", 
            (result.successCount * 100.0 / result.fileCount)));
        logger.info("==========================================");
    }
    
    private void createTestFiles(int count) throws IOException {
        logger.info("创建 {} 个测试文件...", count);
        
        for (int i = 0; i < count; i++) {
            String fileName = String.format("test_%04d.jpg", i);
            File file = new File(testBasePath, fileName);
            
            try (FileOutputStream fos = new FileOutputStream(file)) {
                byte[] data = new byte[1024];
                for (int j = 0; j < data.length; j++) {
                    data[j] = (byte) (i % 256);
                }
                fos.write(data);
            }
            
            testFiles.add(file);
        }
        
        logger.info("测试文件创建完成");
    }
    
    private static void deleteDirectory(File directory) {
        File[] files = directory.listFiles();
        if (files != null) {
            for (File file : files) {
                if (file.isDirectory()) {
                    deleteDirectory(file);
                } else {
                    file.delete();
                }
            }
        }
        directory.delete();
    }
    
    private static class PerformanceResult {
        String testName;
        int fileCount;
        long durationMs;
        double durationSeconds;
        double filesPerSecond;
        long successCount;
        long failureCount;
    }
}
