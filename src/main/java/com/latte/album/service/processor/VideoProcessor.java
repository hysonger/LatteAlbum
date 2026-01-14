package com.latte.album.service.processor;

import com.latte.album.model.MediaFile;
import com.latte.album.service.VideoProcessorService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.util.Set;
import java.util.concurrent.CompletableFuture;

/**
 * 视频处理器 - 处理mp4/avi/mov/mkv等视频格式
 */
@Component
public class VideoProcessor implements MediaProcessor {

    private static final Logger logger = LoggerFactory.getLogger(VideoProcessor.class);

    private static final Set<String> SUPPORTED_EXTENSIONS = Set.of(
        ".mp4", ".avi", ".mov", ".mkv", ".wmv", ".flv", ".webm"
    );

    private final VideoProcessorService videoService;

    public VideoProcessor(VideoProcessorService videoService) {
        this.videoService = videoService;
    }

    @Override
    public MediaType getMediaType() {
        return MediaType.VIDEO;
    }

    @Override
    public boolean supports(File file) {
        if (file == null || !file.isFile()) {
            return false;
        }
        String ext = getFileExtension(file).toLowerCase();
        return SUPPORTED_EXTENSIONS.contains(ext);
    }

    @Override
    public int getPriority() {
        return 10;
    }

    @Override
    public void process(File file, MediaFile mediaFile) {
        videoService.extractVideoMetadata(file, mediaFile);
        try {
            mediaFile.setMimeType(Files.probeContentType(file.toPath()));
        } catch (IOException e) {
            logger.warn("无法确定文件MIME类型: {}", file.getName());
        }
    }

    @Override
    public byte[] generateThumbnail(File file, int width) {
        return videoService.generateThumbnail(file, width);
    }

    @Override
    public CompletableFuture<byte[]> generateThumbnailAsync(File file, int width) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                logger.debug("开始异步生成视频缩略图: {}, width: {}", file.getName(), width);
                byte[] result = videoService.generateThumbnail(file, width);
                logger.debug("异步生成视频缩略图完成: {}, 大小: {} bytes",
                    file.getName(), result.length);
                return result;
            } catch (Exception e) {
                logger.error("异步生成视频缩略图失败: {}", file.getName(), e);
                return new byte[0];
            }
        });
    }

    private String getFileExtension(File file) {
        String name = file.getName();
        int lastDot = name.lastIndexOf('.');
        return lastDot > 0 ? name.substring(lastDot) : "";
    }
}
