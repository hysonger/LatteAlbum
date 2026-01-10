package com.latte.album.service;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;

import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;
import java.io.BufferedReader;
import java.io.File;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Service
public class HeifProcessorService {
    
    private static final Logger logger = LoggerFactory.getLogger(HeifProcessorService.class);
    
    private static final String HEIF_CONVERT_CMD = "heif-convert";
    private static final String HEIF_INFO_CMD = "heif-info";
    private static final String[] HEIF_EXTENSIONS = {".heic", ".heif", ".HEIC", ".HEIF"};
    
    private static final Pattern DIMENSION_PATTERN = Pattern.compile("image:\\s*(\\d+)\\s*[xX×]\\s*(\\d+)");
    
    public boolean isHeifFile(File file) {
        if (file == null || !file.isFile()) {
            return false;
        }
        String fileName = file.getName();
        for (String ext : HEIF_EXTENSIONS) {
            if (fileName.toLowerCase().endsWith(ext.toLowerCase())) {
                return true;
            }
        }
        return false;
    }
    
    public boolean isHeifSupported() {
        try {
            ProcessBuilder pb = new ProcessBuilder(HEIF_CONVERT_CMD, "--version");
            Process process = pb.start();
            int exitCode = process.waitFor();
            return exitCode == 0;
        } catch (IOException | InterruptedException e) {
            logger.warn("HEIF转换工具不可用: {}", e.getMessage());
            return false;
        }
    }
    
    public File convertToJpeg(File heifFile, File outputJpegFile) throws IOException, InterruptedException {
        return convertHeif(heifFile, outputJpegFile, "jpg", null);
    }
    
    public File convertHeif(File heifFile, File outputFile, String format, Integer quality) throws IOException, InterruptedException {
        if (heifFile == null) {
            throw new IllegalArgumentException("HEIF文件不能为null");
        }
        
        if (!isHeifFile(heifFile)) {
            throw new IllegalArgumentException("文件不是HEIF格式: " + heifFile.getName());
        }
        
        if (!isHeifSupported()) {
            throw new IOException("HEIF转换工具不可用，请安装libheif");
        }
        
        List<String> command = new ArrayList<>();
        command.add(HEIF_CONVERT_CMD);
        
        if (quality != null && "jpg".equalsIgnoreCase(format)) {
            command.add("-q");
            command.add(quality.toString());
        }
        
        command.add(heifFile.getAbsolutePath());
        
        if (outputFile != null) {
            command.add(outputFile.getAbsolutePath());
        }
        
        logger.debug("执行HEIF转换命令: {}", String.join(" ", command));
        
        ProcessBuilder pb = new ProcessBuilder(command);
        pb.redirectErrorStream(true);
        
        Process process = pb.start();
        
        BufferedReader reader = new BufferedReader(new InputStreamReader(process.getInputStream()));
        String line;
        while ((line = reader.readLine()) != null) {
            logger.debug("heif-convert输出: {}", line);
        }
        
        int exitCode = process.waitFor();
        
        if (exitCode != 0) {
            throw new IOException("HEIF转换失败，退出码: " + exitCode);
        }
        
        File resultFile = outputFile;
        if (resultFile == null) {
            String baseName = heifFile.getName();
            int dotIndex = baseName.lastIndexOf('.');
            if (dotIndex > 0) {
                baseName = baseName.substring(0, dotIndex);
            }
            resultFile = new File(heifFile.getParentFile(), baseName + ".jpg");
        }
        
        if (!resultFile.exists()) {
            throw new IOException("转换后的文件不存在: " + resultFile.getAbsolutePath());
        }
        
        logger.info("成功转换HEIF文件: {} -> {}", heifFile.getName(), resultFile.getName());
        return resultFile;
    }
    
    public BufferedImage readHeifImage(File heifFile) throws IOException, InterruptedException {
        Path tempFile = null;
        try {
            tempFile = Files.createTempFile("heif-convert-", ".jpg");
            File jpegFile = convertToJpeg(heifFile, tempFile.toFile());
            BufferedImage image = ImageIO.read(jpegFile);
            if (image == null) {
                throw new IOException("无法读取转换后的JPEG图像");
            }
            return image;
        } finally {
            if (tempFile != null) {
                try {
                    Files.deleteIfExists(tempFile);
                } catch (IOException e) {
                    logger.warn("删除临时文件失败: {}", tempFile, e);
                }
            }
        }
    }
    
    public int[] getImageDimensions(File heifFile) throws IOException, InterruptedException {
        if (heifFile == null) {
            throw new IllegalArgumentException("HEIF文件不能为null");
        }
        
        if (!isHeifFile(heifFile)) {
            throw new IllegalArgumentException("文件不是HEIF格式: " + heifFile.getName());
        }
        
        List<String> command = new ArrayList<>();
        command.add(HEIF_INFO_CMD);
        command.add(heifFile.getAbsolutePath());
        
        ProcessBuilder pb = new ProcessBuilder(command);
        pb.redirectErrorStream(true);
        
        Process process = pb.start();
        
        BufferedReader reader = new BufferedReader(new InputStreamReader(process.getInputStream()));
        String line;
        int[] dimensions = null;
        
        while ((line = reader.readLine()) != null) {
            logger.debug("heif-info输出: {}", line);
            Matcher matcher = DIMENSION_PATTERN.matcher(line);
            if (matcher.find()) {
                int width = Integer.parseInt(matcher.group(1));
                int height = Integer.parseInt(matcher.group(2));
                dimensions = new int[]{width, height};
                logger.debug("检测到HEIF图像尺寸: {}x{}", width, height);
            }
        }
        
        int exitCode = process.waitFor();
        
        if (exitCode != 0) {
            throw new IOException("获取HEIF图像尺寸失败，退出码: " + exitCode);
        }
        
        if (dimensions == null) {
            throw new IOException("无法从HEIF文件中提取图像尺寸");
        }
        
        return dimensions;
    }
    
    public byte[] convertToJpegBytes(File heifFile, int quality) throws IOException, InterruptedException {
        Path tempFile = null;
        try {
            tempFile = Files.createTempFile("heif-convert-", ".jpg");
            File jpegFile = convertHeif(heifFile, tempFile.toFile(), "jpg", quality);
            return Files.readAllBytes(jpegFile.toPath());
        } finally {
            if (tempFile != null) {
                try {
                    Files.deleteIfExists(tempFile);
                } catch (IOException e) {
                    logger.warn("删除临时文件失败: {}", tempFile, e);
                }
            }
        }
    }
    
    /**
     * 异步转换HEIF为JPEG字节数组
     * @param heifFile HEIF文件
     * @param quality JPEG质量（0-100）
     * @return CompletableFuture包含JPEG字节数组
     */
    @Async("heifConversionExecutor")
    public CompletableFuture<byte[]> convertToJpegBytesAsync(File heifFile, int quality) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                logger.debug("开始异步转换HEIF为JPEG: {}", heifFile.getName());
                byte[] result = convertToJpegBytes(heifFile, quality);
                logger.debug("HEIF异步转换完成: {}, 大小: {} bytes", 
                    heifFile.getName(), result.length);
                return result;
            } catch (Exception e) {
                logger.error("HEIF异步转换失败: {}", heifFile.getName(), e);
                return new byte[0];
            }
        });
    }
    
    public byte[] generateThumbnail(File heifFile, int width, int quality) throws IOException, InterruptedException {
        Path tempFile = null;
        try {
            tempFile = Files.createTempFile("heif-convert-", ".jpg");
            File jpegFile = convertHeif(heifFile, tempFile.toFile(), "jpg", quality);
            
            net.coobird.thumbnailator.Thumbnails.of(jpegFile)
                    .width(width)
                    .keepAspectRatio(true)
                    .outputFormat("jpg")
                    .outputQuality(quality / 100.0)
                    .toFile(jpegFile);
            
            return Files.readAllBytes(jpegFile.toPath());
        } finally {
            if (tempFile != null) {
                try {
                    Files.deleteIfExists(tempFile);
                } catch (IOException e) {
                    logger.warn("删除临时文件失败: {}", tempFile, e);
                }
            }
        }
    }
}