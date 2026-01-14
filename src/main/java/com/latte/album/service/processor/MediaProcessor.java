package com.latte.album.service.processor;

import com.latte.album.model.MediaFile;
import java.io.File;
import java.util.concurrent.CompletableFuture;

/**
 * 媒体处理器接口 - 策略模式
 * 定义每种媒体格式必须实现的方法
 */
public interface MediaProcessor {

    /**
     * 媒体类型枚举
     */
    enum MediaType {
        IMAGE,   // 常规图片
        VIDEO,   // 视频
        HEIF,    // HEIF/HEIC格式
        RAW      // RAW格式（预留）
    }

    /**
     * 获取处理器支持的媒体类型
     */
    MediaType getMediaType();

    /**
     * 检查文件是否由该处理器支持
     */
    boolean supports(File file);

    /**
     * 处理文件：提取元数据
     */
    void process(File file, MediaFile mediaFile);

    /**
     * 生成缩略图
     * @param file 原始文件
     * @param width 目标宽度（0表示full size，不缩放只转换格式）
     * @return 缩略图字节数组
     */
    byte[] generateThumbnail(File file, int width);

    /**
     * 异步生成缩略图
     * @param file 原始文件
     * @param width 目标宽度
     * @return CompletableFuture包含缩略图字节数组
     */
    CompletableFuture<byte[]> generateThumbnailAsync(File file, int width);

    /**
     * 获取处理器优先级（数值越大优先级越高）
     * 当多个处理器都支持同一文件时，优先级高的优先
     */
    default int getPriority() {
        return 0;
    }
}
