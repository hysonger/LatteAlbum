package com.latte.album.service;

import com.github.benmanes.caffeine.cache.Cache;
import com.github.benmanes.caffeine.cache.Caffeine;
import com.latte.album.config.ApplicationConfig;
import com.latte.album.model.MediaFile;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

import jakarta.annotation.PostConstruct;
import java.io.File;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.concurrent.TimeUnit;

@Service
public class CacheService {
    
    private static final Logger logger = LoggerFactory.getLogger(CacheService.class);
    
    private final ApplicationConfig applicationConfig;
    private final MediaProcessorService mediaProcessorService;
    
    // 内存缓存：使用Caffeine实现
    private Cache<String, byte[]> memoryCache;
    
    @Autowired
    public CacheService(ApplicationConfig applicationConfig, 
                        MediaProcessorService mediaProcessorService) {
        this.applicationConfig = applicationConfig;
        this.mediaProcessorService = mediaProcessorService;
    }
    
    @PostConstruct
    public void init() {
        // 初始化内存缓存
        memoryCache = Caffeine.newBuilder()
                .maximumSize(applicationConfig.isCacheEnabled() ? applicationConfig.getSmallThumbnailSize() : 0)
                .expireAfterWrite(1, TimeUnit.HOURS)
                .build();
        
        // 确保缓存目录存在
        if (applicationConfig.isCacheEnabled()) {
            try {
                Path cacheDir = Paths.get(applicationConfig.getDiskCachePath());
                if (!Files.exists(cacheDir)) {
                    Files.createDirectories(cacheDir);
                }
            } catch (Exception e) {
                logger.error("创建缓存目录失败", e);
            }
        }
    }
    
    /**
     * 管理三级缓存：
     * 1. 内存缓存：热数据（最近访问的缩略图）
     * 2. 磁盘缓存：所有缩略图
     * 3. 数据库缓存：EXIF等元数据
     */
    public byte[] getThumbnail(MediaFile mediaFile, String size) {
        String fileId = mediaFile.getId();
        String cacheKey = fileId + "_" + size;
        
        // 1. 检查内存缓存
        byte[] thumbnail = memoryCache.getIfPresent(cacheKey);
        if (thumbnail != null && thumbnail.length > 0) {
            logger.debug("从内存缓存获取缩略图: {}", cacheKey);
            return thumbnail;
        }
        
        // 2. 检查磁盘缓存
        thumbnail = getFromDiskCache(fileId, size);
        if (thumbnail != null && thumbnail.length > 0) {
            logger.debug("从磁盘缓存获取缩略图: {}", cacheKey);
            // 放入内存缓存
            if (applicationConfig.isCacheEnabled()) {
                memoryCache.put(cacheKey, thumbnail);
            }
            return thumbnail;
        }
        
        // 3. 生成新的缩略图
        thumbnail = generateAndCacheThumbnail(mediaFile, size);
        return thumbnail;
    }
    
    /**
     * 从磁盘缓存获取缩略图
     */
    private byte[] getFromDiskCache(String fileId, String size) {
        if (!applicationConfig.isCacheEnabled()) {
            return new byte[0];
        }
        
        try {
            Path cacheFile = getCacheFilePath(fileId, size);
            if (Files.exists(cacheFile)) {
                return Files.readAllBytes(cacheFile);
            }
        } catch (Exception e) {
            logger.warn("读取磁盘缓存失败", e);
        }
        return new byte[0];
    }
    
    /**
     * 生成并缓存缩略图
     */
    private byte[] generateAndCacheThumbnail(MediaFile mediaFile, String size) {
        try {
            File originalFile = new File(mediaFile.getFilePath());
            if (!originalFile.exists()) {
                return new byte[0];
            }
            
            int width = getThumbnailWidth(size);
            
            // 统一使用异步方法生成缩略图（包括full size）
            byte[] thumbnail = mediaProcessorService.generateThumbnailAsync(originalFile, width).get();
            
            // 保存到磁盘缓存
            if (applicationConfig.isCacheEnabled() && thumbnail.length > 0) {
                saveToDiskCache(mediaFile.getId(), size, thumbnail);
                // 保存到内存缓存
                String cacheKey = mediaFile.getId() + "_" + size;
                memoryCache.put(cacheKey, thumbnail);
            }
            
            return thumbnail;
        } catch (Exception e) {
            logger.error("生成缩略图失败", e);
            return new byte[0];
        }
    }
    
    /**
     * 保存到磁盘缓存
     */
    private void saveToDiskCache(String fileId, String size, byte[] thumbnail) {
        try {
            Path cacheFile = getCacheFilePath(fileId, size);
            Files.write(cacheFile, thumbnail);
        } catch (Exception e) {
            logger.warn("保存磁盘缓存失败", e);
        }
    }
    
    /**
     * 获取缓存文件路径
     */
    private Path getCacheFilePath(String fileId, String size) {
        String cacheDir = applicationConfig.getDiskCachePath();
        String fileName = fileId + "_" + size + ".jpg";
        return Paths.get(cacheDir, fileName);
    }
    
    /**
     * 根据尺寸名称获取缩略图宽度
     */
    private int getThumbnailWidth(String size) {
        switch (size.toLowerCase()) {
            case "small":
                return applicationConfig.getSmallThumbnailSize();
            case "medium":
                return applicationConfig.getMediumThumbnailSize();
            case "large":
                return applicationConfig.getLargeThumbnailSize();
            case "full":
                return 0;
            default:
                return applicationConfig.getSmallThumbnailSize();
        }
    }

    /**
     * 删除指定文件的所有缩略图缓存
     * @param fileId 文件ID
     */
    public void deleteThumbnails(String fileId) {
        if (!applicationConfig.isCacheEnabled()) {
            return;
        }

        String[] sizes = {"small", "medium", "large"};
        for (String size : sizes) {
            try {
                Path cacheFile = getCacheFilePath(fileId, size);
                if (Files.exists(cacheFile)) {
                    Files.delete(cacheFile);
                    logger.info("删除缩略图缓存: {}", cacheFile);
                }
                // 从内存缓存中移除
                memoryCache.invalidate(fileId + "_" + size);
            } catch (Exception e) {
                logger.warn("删除缩略图缓存失败: fileId={}, size={}", fileId, size, e);
            }
        }
    }
}