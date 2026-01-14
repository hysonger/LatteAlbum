package com.latte.album.service;

import com.latte.album.model.MediaFile;
import com.latte.album.service.processor.MediaProcessor;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;

import java.io.File;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

/**
 * 媒体处理服务 - 协调者角色
 * 负责根据文件类型选择合适的处理器，不再包含具体处理逻辑
 */
@Service
public class MediaProcessorService {

    private static final Logger logger = LoggerFactory.getLogger(MediaProcessorService.class);

    private final List<MediaProcessor> processors;

    public MediaProcessorService(List<MediaProcessor> processors) {
        this.processors = processors;
        // 按优先级排序，优先级高的处理器优先匹配
        this.processors.sort(Comparator.comparingInt(MediaProcessor::getPriority).reversed());
        logger.info("注册了 {} 个媒体处理器: {}",
            processors.size(),
            processors.stream().map(p -> p.getMediaType().name()).toList());
    }

    /**
     * 处理图片
     */
    public void processImage(File imageFile, MediaFile mediaFile) {
        findProcessor(imageFile).ifPresent(processor -> {
            logger.debug("使用 {} 处理器处理图片: {}",
                processor.getClass().getSimpleName(), imageFile.getName());
            processor.process(imageFile, mediaFile);
        });
    }

    /**
     * 处理视频
     */
    public void processVideo(File videoFile, MediaFile mediaFile) {
        findProcessor(videoFile).ifPresent(processor -> {
            logger.debug("使用 {} 处理器处理视频: {}",
                processor.getClass().getSimpleName(), videoFile.getName());
            processor.process(videoFile, mediaFile);
        });
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

        return findProcessor(originalFile)
            .map(processor -> processor.generateThumbnail(originalFile, width))
            .orElse(new byte[0]);
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

    /**
     * 查找合适的处理器（优先级最高者）
     */
    private Optional<MediaProcessor> findProcessor(File file) {
        return processors.stream()
            .filter(p -> p.supports(file))
            .findFirst();
    }
}
