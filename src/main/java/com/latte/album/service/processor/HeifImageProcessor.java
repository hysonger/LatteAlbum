package com.latte.album.service.processor;

import com.latte.album.config.ApplicationConfig;
import com.latte.album.service.HeifProcessorService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import java.io.File;
import java.io.IOException;
import java.util.Set;
import java.util.concurrent.CompletableFuture;

/**
 * HEIF图片处理器 - 处理.heic/.heif格式
 */
@Component
public class HeifImageProcessor extends AbstractImageProcessor {

    private static final Set<String> SUPPORTED_EXTENSIONS = Set.of(
        ".heic", ".heif"
    );

    private final HeifProcessorService heifService;

    @Autowired
    public HeifImageProcessor(ApplicationConfig config, HeifProcessorService heifService) {
        super(config);
        this.heifService = heifService;
    }

    @Override
    public MediaType getMediaType() {
        return MediaType.HEIF;
    }

    @Override
    public boolean supports(File file) {
        return heifService.isHeifFile(file);
    }

    @Override
    public int getPriority() {
        return 100; // HEIF优先级高于常规图片
    }

    @Override
    protected int[] getImageDimensions(File file) throws IOException, InterruptedException {
        return heifService.getImageDimensions(file);
    }

    @Override
    public byte[] generateThumbnail(File file, int width) {
        try {
            if (!heifService.isHeifSupported()) {
                logger.warn("HEIF转换工具不可用，无法生成缩略图: {}", file.getName());
                return new byte[0];
            }

            int quality = (int) (config.getThumbnailQuality() * 100);

            if (width == 0) {
                return heifService.convertToJpegBytes(file, quality);
            }
            return heifService.generateThumbnail(file, width, quality);
        } catch (IOException | InterruptedException e) {
            logger.error("生成HEIF缩略图失败: {}", file.getName(), e);
            Thread.currentThread().interrupt();
            return new byte[0];
        }
    }

    @Override
    public CompletableFuture<byte[]> generateThumbnailAsync(File file, int width) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                logger.debug("开始异步生成HEIF缩略图: {}, width: {}", file.getName(), width);
                byte[] result = generateThumbnail(file, width);
                logger.debug("异步生成HEIF缩略图完成: {}, 大小: {} bytes",
                    file.getName(), result.length);
                return result;
            } catch (Exception e) {
                logger.error("异步生成HEIF缩略图失败: {}", file.getName(), e);
                return new byte[0];
            }
        });
    }
}
