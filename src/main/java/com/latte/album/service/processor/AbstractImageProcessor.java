package com.latte.album.service.processor;

import com.drew.imaging.ImageMetadataReader;
import com.drew.imaging.ImageProcessingException;
import com.drew.metadata.Metadata;
import com.drew.metadata.exif.ExifIFD0Directory;
import com.drew.metadata.exif.ExifSubIFDDirectory;
import com.latte.album.config.ApplicationConfig;
import com.latte.album.model.MediaFile;
import net.coobird.thumbnailator.Thumbnails;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

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

/**
 * 图片处理器抽象基类 - 模板方法模式
 * 提供EXIF解析的通用逻辑，消除代码重复
 */
public abstract class AbstractImageProcessor implements MediaProcessor {

    protected final Logger logger = LoggerFactory.getLogger(getClass());

    protected final ApplicationConfig config;

    protected AbstractImageProcessor(ApplicationConfig config) {
        this.config = config;
    }

    @Override
    public void process(File file, MediaFile mediaFile) {
        try {
            // 提取EXIF元数据（模板方法）
            Metadata metadata = ImageMetadataReader.readMetadata(file);
            extractExifMetadata(metadata, mediaFile);

            // 获取图片尺寸（由子类实现）
            int[] dimensions = getImageDimensions(file);
            if (dimensions != null && dimensions.length == 2) {
                mediaFile.setWidth(dimensions[0]);
                mediaFile.setHeight(dimensions[1]);
            }

            // 设置MIME类型
            mediaFile.setMimeType(Files.probeContentType(file.toPath()));

        } catch (ImageProcessingException e) {
            logger.warn("无法读取图片元数据: {}", file.getName(), e);
        } catch (IOException | InterruptedException e) {
            logger.warn("无法获取图片尺寸: {}", file.getName(), e);
            Thread.currentThread().interrupt();
        }
    }

    /**
     * 获取图片尺寸 - 由子类实现
     */
    protected abstract int[] getImageDimensions(File file) throws IOException, InterruptedException;

    /**
     * 通用EXIF解析逻辑 - 消除代码重复
     */
    protected void extractExifMetadata(Metadata metadata, MediaFile mediaFile) {
        ExifIFD0Directory ifd0Directory = metadata.getFirstDirectoryOfType(ExifIFD0Directory.class);
        ExifSubIFDDirectory exifDirectory = metadata.getFirstDirectoryOfType(ExifSubIFDDirectory.class);

        // 提取相机厂商
        String cameraMake = extractCameraMake(ifd0Directory, exifDirectory);
        if (cameraMake != null) {
            mediaFile.setCameraMake(cameraMake);
        }

        // 提取相机型号
        String cameraModel = extractCameraModel(ifd0Directory, exifDirectory);
        if (cameraModel != null) {
            mediaFile.setCameraModel(cameraModel);
        }

        // 提取拍摄时间和曝光参数
        if (exifDirectory != null) {
            extractExifTime(exifDirectory, mediaFile);
            extractExposureSettings(exifDirectory, mediaFile);
        }
    }

    private String extractCameraMake(ExifIFD0Directory ifd0, ExifSubIFDDirectory exif) {
        if (ifd0 != null) {
            String make = ifd0.getString(ExifIFD0Directory.TAG_MAKE);
            if (make != null) return make;
        }
        if (exif != null) {
            return exif.getString(ExifSubIFDDirectory.TAG_MAKE);
        }
        return null;
    }

    private String extractCameraModel(ExifIFD0Directory ifd0, ExifSubIFDDirectory exif) {
        if (ifd0 != null) {
            String model = ifd0.getString(ExifIFD0Directory.TAG_MODEL);
            if (model != null) return model;
        }
        if (exif != null) {
            return exif.getString(ExifSubIFDDirectory.TAG_MODEL);
        }
        return null;
    }

    private void extractExifTime(ExifSubIFDDirectory exif, MediaFile mediaFile) {
        // 提取拍摄时间 - 使用 UTC 时区避免时区转换问题
        Date dateTimeOriginal = exif.getDate(ExifSubIFDDirectory.TAG_DATETIME_ORIGINAL);
        if (dateTimeOriginal != null) {
            mediaFile.setExifTimestamp(LocalDateTime.ofInstant(
                dateTimeOriginal.toInstant(), ZoneOffset.UTC));
            // 提取时区偏移量 OffsetTimeOriginal (Tag 0x9010)
            String offsetTimeOriginal = exif.getString(0x9010);
            if (offsetTimeOriginal != null && !offsetTimeOriginal.isEmpty()) {
                mediaFile.setExifTimezoneOffset(offsetTimeOriginal);
            }
        } else {
            // 尝试使用普通日期时间标签
            Date dateTime = exif.getDate(ExifSubIFDDirectory.TAG_DATETIME);
            if (dateTime != null) {
                mediaFile.setExifTimestamp(LocalDateTime.ofInstant(
                    dateTime.toInstant(), ZoneOffset.UTC));
                // 提取时区偏移量 OffsetTime (Tag 0x9012)
                String offsetTime = exif.getString(0x9012);
                if (offsetTime != null && !offsetTime.isEmpty()) {
                    mediaFile.setExifTimezoneOffset(offsetTime);
                }
            }
        }
    }

    private void extractExposureSettings(ExifSubIFDDirectory exif, MediaFile mediaFile) {
        // 光圈值
        String aperture = exif.getString(ExifSubIFDDirectory.TAG_FNUMBER);
        if (aperture != null) {
            mediaFile.setAperture(aperture);
        }

        // 快门速度
        String exposureTime = exif.getString(ExifSubIFDDirectory.TAG_EXPOSURE_TIME);
        if (exposureTime != null) {
            mediaFile.setExposureTime(exposureTime);
        }

        // ISO
        Integer iso = exif.getInteger(ExifSubIFDDirectory.TAG_ISO_EQUIVALENT);
        if (iso != null) {
            mediaFile.setIso(iso);
        }

        // 焦距
        String focalLength = exif.getString(ExifSubIFDDirectory.TAG_FOCAL_LENGTH);
        if (focalLength != null) {
            mediaFile.setFocalLength(focalLength);
        }
    }

    /**
     * 通用图片尺寸获取 - 使用ImageIO
     */
    protected int[] getImageDimensionsWithImageIO(File file) throws IOException {
        BufferedImage image = ImageIO.read(file);
        if (image != null) {
            return new int[]{image.getWidth(), image.getHeight()};
        }
        return null;
    }

    @Override
    public byte[] generateThumbnail(File file, int width) {
        if (file == null || !file.exists()) {
            logger.warn("文件不存在，无法生成缩略图: {}", file);
            return new byte[0];
        }

        if (file.length() == 0) {
            logger.warn("文件为空，无法生成缩略图: {}", file.getName());
            return new byte[0];
        }

        try {
            if (width == 0) {
                return Files.readAllBytes(file.toPath());
            }

            ByteArrayOutputStream outputStream = new ByteArrayOutputStream();
            Thumbnails.of(file)
                    .width(width)
                    .keepAspectRatio(true)
                    .outputFormat("jpg")
                    .outputQuality(config.getThumbnailQuality())
                    .toOutputStream(outputStream);
            return outputStream.toByteArray();
        } catch (IOException e) {
            logger.error("生成缩略图失败: {} (大小: {} bytes)",
                file.getName(), file.length(), e);
            return new byte[0];
        }
    }

    @Override
    public CompletableFuture<byte[]> generateThumbnailAsync(File file, int width) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                logger.debug("开始异步生成缩略图: {}, width: {}", file.getName(), width);
                byte[] result = generateThumbnail(file, width);
                logger.debug("异步生成缩略图完成: {}, 大小: {} bytes",
                    file.getName(), result.length);
                return result;
            } catch (Exception e) {
                logger.error("异步生成缩略图失败: {}", file.getName(), e);
                return new byte[0];
            }
        });
    }
}
