package com.latte.album.service.processor;

import com.latte.album.config.ApplicationConfig;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import java.io.File;
import java.io.IOException;
import java.util.Set;

/**
 * 标准图片处理器 - 处理jpg/jpeg/png/gif/bmp/webp/tiff等常规格式
 */
@Component
public class StandardImageProcessor extends AbstractImageProcessor {

    private static final Set<String> SUPPORTED_EXTENSIONS = Set.of(
        ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp", ".tiff"
    );

    public StandardImageProcessor(ApplicationConfig config) {
        super(config);
    }

    @Override
    public MediaType getMediaType() {
        return MediaType.IMAGE;
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
        return 10; // 基础处理器，优先级较低
    }

    @Override
    protected int[] getImageDimensions(File file) throws IOException {
        return getImageDimensionsWithImageIO(file);
    }

    private String getFileExtension(File file) {
        String name = file.getName();
        int lastDot = name.lastIndexOf('.');
        return lastDot > 0 ? name.substring(lastDot) : "";
    }
}
