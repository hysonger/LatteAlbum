package com.latte.album.service;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.springframework.test.util.ReflectionTestUtils;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.*;

class HeifProcessorServiceTest {
    
    private HeifProcessorService heifProcessorService;
    
    @TempDir
    Path tempDir;
    
    @BeforeEach
    void setUp() {
        heifProcessorService = new HeifProcessorService();
    }
    
    @Test
    void testIsHeifFile() throws IOException {
        File heicFile = tempDir.resolve("test.heic").toFile();
        File heifFile = tempDir.resolve("test.heif").toFile();
        File jpegFile = tempDir.resolve("test.jpg").toFile();
        File pngFile = tempDir.resolve("test.png").toFile();
        
        Files.write(heicFile.toPath(), new byte[]{0, 1, 2, 3});
        Files.write(heifFile.toPath(), new byte[]{0, 1, 2, 3});
        Files.write(jpegFile.toPath(), new byte[]{0, 1, 2, 3});
        Files.write(pngFile.toPath(), new byte[]{0, 1, 2, 3});
        
        assertTrue(heifProcessorService.isHeifFile(heicFile));
        assertTrue(heifProcessorService.isHeifFile(heifFile));
        assertFalse(heifProcessorService.isHeifFile(jpegFile));
        assertFalse(heifProcessorService.isHeifFile(pngFile));
    }
    
    @Test
    void testIsHeifFileWithNull() {
        assertFalse(heifProcessorService.isHeifFile(null));
    }
    
    @Test
    void testIsHeifFileWithNonExistentFile() {
        File nonExistentFile = new File("/non/existent/path.heic");
        assertFalse(heifProcessorService.isHeifFile(nonExistentFile));
    }
    
    @Test
    void testIsHeifSupported() {
        boolean supported = heifProcessorService.isHeifSupported();
        assertTrue(supported, "HEIF转换工具应该可用（libheif已安装）");
    }
    
    @Test
    void testIsHeifFileWithUppercaseExtensions() throws IOException {
        File heicUpperFile = tempDir.resolve("test.HEIC").toFile();
        File heifUpperFile = tempDir.resolve("test.HEIF").toFile();
        
        Files.write(heicUpperFile.toPath(), new byte[]{0, 1, 2, 3});
        Files.write(heifUpperFile.toPath(), new byte[]{0, 1, 2, 3});
        
        assertTrue(heifProcessorService.isHeifFile(heicUpperFile));
        assertTrue(heifProcessorService.isHeifFile(heifUpperFile));
    }
    
    @Test
    void testIsHeifFileWithMixedCaseExtensions() throws IOException {
        File heicMixedFile = tempDir.resolve("test.HeIc").toFile();
        File heifMixedFile = tempDir.resolve("test.HeIf").toFile();
        
        Files.write(heicMixedFile.toPath(), new byte[]{0, 1, 2, 3});
        Files.write(heifMixedFile.toPath(), new byte[]{0, 1, 2, 3});
        
        assertTrue(heifProcessorService.isHeifFile(heicMixedFile));
        assertTrue(heifProcessorService.isHeifFile(heifMixedFile));
    }
    
    @Test
    void testConvertHeifWithNonHeifFile() throws IOException {
        File jpegFile = tempDir.resolve("test.jpg").toFile();
        Files.write(jpegFile.toPath(), new byte[]{0, 1, 2, 3});
        
        assertThrows(IllegalArgumentException.class, () -> {
            heifProcessorService.convertToJpeg(jpegFile, tempDir.resolve("output.jpg").toFile());
        });
    }
    
    @Test
    void testConvertHeifWithNullFile() {
        assertThrows(IllegalArgumentException.class, () -> {
            heifProcessorService.convertToJpeg(null, tempDir.resolve("output.jpg").toFile());
        });
    }
    
    @Test
    void testConvertHeifWithNonExistentFile() {
        File nonExistentFile = new File("/non/existent/path.heic");
        assertThrows(IllegalArgumentException.class, () -> {
            heifProcessorService.convertToJpeg(nonExistentFile, tempDir.resolve("output.jpg").toFile());
        });
    }
    
    @Test
    void testGenerateThumbnailWithNonHeifFile() throws IOException {
        File jpegFile = tempDir.resolve("test.jpg").toFile();
        Files.write(jpegFile.toPath(), new byte[]{0, 1, 2, 3});
        
        assertThrows(IllegalArgumentException.class, () -> {
            heifProcessorService.generateThumbnail(jpegFile, 200, 85);
        });
    }
    
    @Test
    void testGenerateThumbnailWithNullFile() {
        assertThrows(IllegalArgumentException.class, () -> {
            heifProcessorService.generateThumbnail(null, 200, 85);
        });
    }
    
    @Test
    void testGetImageDimensionsWithNonHeifFile() throws IOException {
        File jpegFile = tempDir.resolve("test.jpg").toFile();
        Files.write(jpegFile.toPath(), new byte[]{0, 1, 2, 3});
        
        assertThrows(IllegalArgumentException.class, () -> {
            heifProcessorService.getImageDimensions(jpegFile);
        });
    }
    
    @Test
    void testConvertToJpegBytesWithNonHeifFile() throws IOException {
        File jpegFile = tempDir.resolve("test.jpg").toFile();
        Files.write(jpegFile.toPath(), new byte[]{0, 1, 2, 3});
        
        assertThrows(IllegalArgumentException.class, () -> {
            heifProcessorService.convertToJpegBytes(jpegFile, 85);
        });
    }
    
    @Test
    void testReadHeifImageWithNonHeifFile() throws IOException {
        File jpegFile = tempDir.resolve("test.jpg").toFile();
        Files.write(jpegFile.toPath(), new byte[]{0, 1, 2, 3});
        
        assertThrows(IllegalArgumentException.class, () -> {
            heifProcessorService.readHeifImage(jpegFile);
        });
    }
}