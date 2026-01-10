package com.latte.album.service;

import com.drew.imaging.ImageMetadataReader;
import com.drew.imaging.ImageProcessingException;
import com.drew.metadata.Metadata;
import com.drew.metadata.exif.ExifIFD0Directory;
import com.drew.metadata.exif.ExifSubIFDDirectory;
import com.latte.album.model.MediaFile;
import net.coobird.thumbnailator.Thumbnails;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;

import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;
import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.util.Date;
import java.util.concurrent.CompletableFuture;

@Service
public class MediaProcessorService {
    
    private static final Logger logger = LoggerFactory.getLogger(MediaProcessorService.class);
    
    private final HeifProcessorService heifProcessorService;
    private final VideoProcessorService videoProcessorService;
    private final com.latte.album.config.ApplicationConfig applicationConfig;
    
    public MediaProcessorService(HeifProcessorService heifProcessorService,
                               VideoProcessorService videoProcessorService,
                               com.latte.album.config.ApplicationConfig applicationConfig) {
        this.heifProcessorService = heifProcessorService;
        this.videoProcessorService = videoProcessorService;
        this.applicationConfig = applicationConfig;
    }
    
    /**
     * 处理图片：生成缩略图，提取EXIF
     */
    public void processImage(File imageFile, MediaFile mediaFile) {
        try {
            // 检查是否为HEIF格式
            if (heifProcessorService.isHeifFile(imageFile)) {
                // 处理HEIF格式图片
                extractHeifMetadata(imageFile, mediaFile);
            } else {
                // 处理普通格式图片
                extractImageMetadata(imageFile, mediaFile);
            }
            
            // 生成缩略图标记（实际生成将在请求时进行）
            mediaFile.setThumbnailGenerated(false);
            
            logger.debug("成功处理图片: {}", imageFile.getName());
        } catch (Exception e) {
            logger.error("处理图片失败: " + imageFile.getName(), e);
        }
    }
    
    /**
     * 处理视频：生成预览图，提取元数据
     */
    public void processVideo(File videoFile, MediaFile mediaFile) {
        try {
            videoProcessorService.extractVideoMetadata(videoFile, mediaFile);
            
            mediaFile.setMimeType(Files.probeContentType(videoFile.toPath()));
            mediaFile.setThumbnailGenerated(false);
            
            logger.debug("成功处理视频: {}", videoFile.getName());
        } catch (Exception e) {
            logger.error("处理视频失败: " + videoFile.getName(), e);
        }
    }
    
    /**
     * 提取图片EXIF信息
     * @param imageFile 图片文件
     * @param mediaFile 媒体文件实体
     */
    private void extractImageMetadata(File imageFile, MediaFile mediaFile) {
        try {
            Metadata metadata = ImageMetadataReader.readMetadata(imageFile);
            
            // 获取EXIF IFD0目录信息（包含相机厂商和型号）
            ExifIFD0Directory ifd0Directory = metadata.getFirstDirectoryOfType(ExifIFD0Directory.class);
            
            // 获取EXIF子目录信息（包含拍摄参数）
            ExifSubIFDDirectory exifDirectory = metadata.getFirstDirectoryOfType(ExifSubIFDDirectory.class);
            
            // 提取相机厂商（优先从IFD0目录获取）
            String cameraMake = null;
            if (ifd0Directory != null) {
                cameraMake = ifd0Directory.getString(ExifIFD0Directory.TAG_MAKE);
            }
            if (cameraMake == null && exifDirectory != null) {
                cameraMake = exifDirectory.getString(ExifSubIFDDirectory.TAG_MAKE);
            }
            if (cameraMake != null) {
                mediaFile.setCameraMake(cameraMake);
            }
            
            // 提取相机型号（优先从IFD0目录获取）
            String cameraModel = null;
            if (ifd0Directory != null) {
                cameraModel = ifd0Directory.getString(ExifIFD0Directory.TAG_MODEL);
            }
            if (cameraModel == null && exifDirectory != null) {
                cameraModel = exifDirectory.getString(ExifSubIFDDirectory.TAG_MODEL);
            }
            if (cameraModel != null) {
                mediaFile.setCameraModel(cameraModel);
            }
            
            if (exifDirectory != null) {
                // 提取拍摄时间 - 使用 UTC 时区避免时区转换问题
                Date dateTimeOriginal = exifDirectory.getDate(ExifSubIFDDirectory.TAG_DATETIME_ORIGINAL);
                if (dateTimeOriginal != null) {
                    mediaFile.setExifTimestamp(LocalDateTime.ofInstant(
                        dateTimeOriginal.toInstant(), ZoneOffset.UTC));
                    // 提取时区偏移量 OffsetTimeOriginal (Tag 0x9010)
                    String offsetTimeOriginal = exifDirectory.getString(0x9010);
                    if (offsetTimeOriginal != null && !offsetTimeOriginal.isEmpty()) {
                        mediaFile.setExifTimezoneOffset(offsetTimeOriginal);
                    }
                } else {
                    // 尝试使用普通日期时间标签
                    Date dateTime = exifDirectory.getDate(ExifSubIFDDirectory.TAG_DATETIME);
                    if (dateTime != null) {
                        mediaFile.setExifTimestamp(LocalDateTime.ofInstant(
                            dateTime.toInstant(), ZoneOffset.UTC));
                        // 提取时区偏移量 OffsetTime (Tag 0x9012)
                        String offsetTime = exifDirectory.getString(0x9012);
                        if (offsetTime != null && !offsetTime.isEmpty()) {
                            mediaFile.setExifTimezoneOffset(offsetTime);
                        }
                    }
                }
                
                // 提取光圈值
                String aperture = exifDirectory.getString(ExifSubIFDDirectory.TAG_FNUMBER);
                if (aperture != null) {
                    mediaFile.setAperture(aperture);
                }
                
                // 提取快门速度
                String exposureTime = exifDirectory.getString(ExifSubIFDDirectory.TAG_EXPOSURE_TIME);
                if (exposureTime != null) {
                    mediaFile.setExposureTime(exposureTime);
                }
                
                // 提取ISO
                Integer iso = exifDirectory.getInteger(ExifSubIFDDirectory.TAG_ISO_EQUIVALENT);
                if (iso != null) {
                    mediaFile.setIso(iso);
                }
                
                // 提取焦距
                String focalLength = exifDirectory.getString(ExifSubIFDDirectory.TAG_FOCAL_LENGTH);
                if (focalLength != null) {
                    mediaFile.setFocalLength(focalLength);
                }
            }
            
            // 设置MIME类型
            mediaFile.setMimeType(Files.probeContentType(imageFile.toPath()));
            
            // 获取图片尺寸
            BufferedImage image = ImageIO.read(imageFile);
            if (image != null) {
                mediaFile.setWidth(image.getWidth());
                mediaFile.setHeight(image.getHeight());
            }
            
        } catch (ImageProcessingException | IOException e) {
            logger.warn("无法读取图片元数据: {}", imageFile.getName(), e);
        }
    }
    
    /**
     * 提取HEIF图片元数据
     * @param heifFile HEIF文件
     * @param mediaFile 媒体文件实体
     */
    private void extractHeifMetadata(File heifFile, MediaFile mediaFile) {
        try {
            // 尝试使用metadata-extractor读取HEIF元数据
            Metadata metadata = ImageMetadataReader.readMetadata(heifFile);
            
            // 获取EXIF IFD0目录信息（包含相机厂商和型号）
            ExifIFD0Directory ifd0Directory = metadata.getFirstDirectoryOfType(ExifIFD0Directory.class);
            
            // 获取EXIF子目录信息（包含拍摄参数）
            ExifSubIFDDirectory exifDirectory = metadata.getFirstDirectoryOfType(ExifSubIFDDirectory.class);
            
            // 提取相机厂商（优先从IFD0目录获取）
            String cameraMake = null;
            if (ifd0Directory != null) {
                cameraMake = ifd0Directory.getString(ExifIFD0Directory.TAG_MAKE);
            }
            if (cameraMake == null && exifDirectory != null) {
                cameraMake = exifDirectory.getString(ExifSubIFDDirectory.TAG_MAKE);
            }
            if (cameraMake != null) {
                mediaFile.setCameraMake(cameraMake);
            }
            
            // 提取相机型号（优先从IFD0目录获取）
            String cameraModel = null;
            if (ifd0Directory != null) {
                cameraModel = ifd0Directory.getString(ExifIFD0Directory.TAG_MODEL);
            }
            if (cameraModel == null && exifDirectory != null) {
                cameraModel = exifDirectory.getString(ExifSubIFDDirectory.TAG_MODEL);
            }
            if (cameraModel != null) {
                mediaFile.setCameraModel(cameraModel);
            }
            
            if (exifDirectory != null) {
                // 提取拍摄时间 - 使用 UTC 时区避免时区转换问题
                Date dateTimeOriginal = exifDirectory.getDate(ExifSubIFDDirectory.TAG_DATETIME_ORIGINAL);
                if (dateTimeOriginal != null) {
                    mediaFile.setExifTimestamp(LocalDateTime.ofInstant(
                        dateTimeOriginal.toInstant(), ZoneOffset.UTC));
                    // 提取时区偏移量 OffsetTimeOriginal (Tag 0x9010)
                    String offsetTimeOriginal = exifDirectory.getString(0x9010);
                    if (offsetTimeOriginal != null && !offsetTimeOriginal.isEmpty()) {
                        mediaFile.setExifTimezoneOffset(offsetTimeOriginal);
                    }
                } else {
                    // 尝试使用普通日期时间标签
                    Date dateTime = exifDirectory.getDate(ExifSubIFDDirectory.TAG_DATETIME);
                    if (dateTime != null) {
                        mediaFile.setExifTimestamp(LocalDateTime.ofInstant(
                            dateTime.toInstant(), ZoneOffset.UTC));
                        // 提取时区偏移量 OffsetTime (Tag 0x9012)
                        String offsetTime = exifDirectory.getString(0x9012);
                        if (offsetTime != null && !offsetTime.isEmpty()) {
                            mediaFile.setExifTimezoneOffset(offsetTime);
                        }
                    }
                }
                
                // 提取光圈值
                String aperture = exifDirectory.getString(ExifSubIFDDirectory.TAG_FNUMBER);
                if (aperture != null) {
                    mediaFile.setAperture(aperture);
                }
                
                // 提取快门速度
                String exposureTime = exifDirectory.getString(ExifSubIFDDirectory.TAG_EXPOSURE_TIME);
                if (exposureTime != null) {
                    mediaFile.setExposureTime(exposureTime);
                }
                
                // 提取ISO
                Integer iso = exifDirectory.getInteger(ExifSubIFDDirectory.TAG_ISO_EQUIVALENT);
                if (iso != null) {
                    mediaFile.setIso(iso);
                }
                
                // 提取焦距
                String focalLength = exifDirectory.getString(ExifSubIFDDirectory.TAG_FOCAL_LENGTH);
                if (focalLength != null) {
                    mediaFile.setFocalLength(focalLength);
                }
            }
            
            // 设置MIME类型
            mediaFile.setMimeType("image/heif");
            
            // 获取HEIF图片尺寸
            int[] dimensions = heifProcessorService.getImageDimensions(heifFile);
            if (dimensions != null && dimensions.length == 2) {
                mediaFile.setWidth(dimensions[0]);
                mediaFile.setHeight(dimensions[1]);
            }
            
        } catch (ImageProcessingException e) {
            logger.warn("无法读取HEIF元数据: {}", heifFile.getName(), e);
        } catch (IOException | InterruptedException e) {
            logger.warn("无法获取HEIF图片尺寸: {}", heifFile.getName(), e);
        }
    }
    
    /**
     * 生成缩略图
     * @param originalFile 原始文件
     * @param width 目标宽度（0表示full size，不缩放只转换格式）
     * @return 缩略图字节数组
     */
    public byte[] generateThumbnail(File originalFile, int width) {
        if (originalFile == null || !originalFile.exists()) {
            logger.warn("文件不存在，无法生成缩略图: {}", originalFile);
            return new byte[0];
        }
        
        if (originalFile.length() == 0) {
            logger.warn("文件为空，无法生成缩略图: {}", originalFile.getName());
            return new byte[0];
        }
        
        try {
            int quality = (int) (applicationConfig.getThumbnailQuality() * 100);
            
            String fileName = originalFile.getName().toLowerCase();
            
            if (fileName.endsWith(".mp4") || fileName.endsWith(".mov") || 
                fileName.endsWith(".avi") || fileName.endsWith(".mkv") ||
                fileName.endsWith(".wmv") || fileName.endsWith(".flv") ||
                fileName.endsWith(".webm")) {
                return videoProcessorService.generateThumbnail(originalFile, width);
            }
            
            if (heifProcessorService.isHeifFile(originalFile)) {
                if (!heifProcessorService.isHeifSupported()) {
                    logger.warn("HEIF转换工具不可用，无法生成缩略图: {}", originalFile.getName());
                    return new byte[0];
                }
                
                if (width == 0) {
                    return heifProcessorService.convertToJpegBytes(originalFile, quality);
                } else {
                    return heifProcessorService.generateThumbnail(originalFile, width, quality);
                }
            }
            
            if (width == 0) {
                return Files.readAllBytes(originalFile.toPath());
            }
            
            ByteArrayOutputStream outputStream = new ByteArrayOutputStream();
            Thumbnails.of(originalFile)
                    .width(width)
                    .keepAspectRatio(true)
                    .outputFormat("jpg")
                    .outputQuality(applicationConfig.getThumbnailQuality())
                    .toOutputStream(outputStream);
            return outputStream.toByteArray();
        } catch (IOException e) {
            logger.error("生成缩略图失败: {} (大小: {} bytes)", 
                originalFile.getName(), originalFile.length(), e);
            return new byte[0];
        } catch (InterruptedException e) {
            logger.error("生成HEIF缩略图被中断: {}", originalFile.getName(), e);
            Thread.currentThread().interrupt();
            return new byte[0];
        }
    }
    
    /**
     * 异步生成缩略图
     * @param originalFile 原始文件
     * @param width 目标宽度（0表示full size，不缩放只转换格式）
     * @return CompletableFuture包含缩略图字节数组
     */
    @Async("thumbnailGenerationExecutor")
    public CompletableFuture<byte[]> generateThumbnailAsync(File originalFile, int width) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                logger.debug("开始异步生成缩略图: {}, width: {}", originalFile.getName(), width);
                byte[] result = generateThumbnail(originalFile, width);
                logger.debug("异步生成缩略图完成: {}, 大小: {} bytes", 
                    originalFile.getName(), result.length);
                return result;
            } catch (Exception e) {
                logger.error("异步生成缩略图失败: {}", originalFile.getName(), e);
                return new byte[0];
            }
        });
    }
}