package com.latte.album.config;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Configuration;

@Configuration
public class ApplicationConfig {
    
    @Value("${album.base-path}")
    private String basePath;
    
    @Value("${album.thumbnail.small:200}")
    private int smallThumbnailSize;
    
    @Value("${album.thumbnail.medium:500}")
    private int mediumThumbnailSize;
    
    @Value("${album.thumbnail.large:1200}")
    private int largeThumbnailSize;
    
    @Value("${album.thumbnail.quality:0.85}")
    private double thumbnailQuality;
    
    @Value("${album.cache.enabled:true}")
    private boolean cacheEnabled;
    
    @Value("${album.cache.disk-path:/tmp/latte-album-cache}")
    private String diskCachePath;
    
    @Value("${album.scan.enabled:true}")
    private boolean scanEnabled;
    
    @Value("${album.scan.parallel.enabled:true}")
    private boolean parallelScanEnabled;
    
    @Value("${album.scan.parallel.thread-pool-size:0}")
    private int parallelThreadPoolSize;
    
    @Value("${album.scan.parallel.batch-size:50}")
    private int parallelBatchSize;
    
    @Value("${album.async.thumbnail.core-pool-size:0}")
    private int thumbnailCorePoolSize;
    
    @Value("${album.async.thumbnail.max-pool-size:0}")
    private int thumbnailMaxPoolSize;
    
    @Value("${album.async.thumbnail.queue-capacity:200}")
    private int thumbnailQueueCapacity;
    
    @Value("${album.async.heif-conversion.core-pool-size:0}")
    private int heifCorePoolSize;
    
    @Value("${album.async.heif-conversion.max-pool-size:0}")
    private int heifMaxPoolSize;
    
    @Value("${album.async.heif-conversion.queue-capacity:100}")
    private int heifQueueCapacity;

    @Value("${album.async.video-processing.core-pool-size:0}")
    private int videoCorePoolSize;

    @Value("${album.async.video-processing.max-pool-size:0}")
    private int videoMaxPoolSize;

    @Value("${album.async.video-processing.queue-capacity:50}")
    private int videoQueueCapacity;

    @Value("${album.video.thumbnail-time-offset:1.0}")
    private double videoThumbnailTimeOffset;

    @Value("${album.video.thumbnail-duration:0.1}")
    private double videoThumbnailDuration;

    public String getBasePath() {
        return basePath;
    }

    public int getSmallThumbnailSize() {
        return smallThumbnailSize;
    }

    public int getMediumThumbnailSize() {
        return mediumThumbnailSize;
    }

    public int getLargeThumbnailSize() {
        return largeThumbnailSize;
    }
    
    public double getThumbnailQuality() {
        return thumbnailQuality;
    }
    
    public boolean isCacheEnabled() {
        return cacheEnabled;
    }
    
    public String getDiskCachePath() {
        return diskCachePath;
    }
    
    public boolean isScanEnabled() {
        return scanEnabled;
    }
    
    public boolean isParallelScanEnabled() {
        return parallelScanEnabled;
    }
    
    public int getParallelThreadPoolSize() {
        return parallelThreadPoolSize;
    }
    
    public int getParallelBatchSize() {
        return parallelBatchSize;
    }
    
    public int getThumbnailCorePoolSize() {
        return thumbnailCorePoolSize;
    }
    
    public int getThumbnailMaxPoolSize() {
        return thumbnailMaxPoolSize;
    }
    
    public int getThumbnailQueueCapacity() {
        return thumbnailQueueCapacity;
    }
    
    public int getHeifCorePoolSize() {
        return heifCorePoolSize;
    }
    
    public int getHeifMaxPoolSize() {
        return heifMaxPoolSize;
    }
    
    public int getHeifQueueCapacity() {
        return heifQueueCapacity;
    }

    public int getVideoCorePoolSize() {
        return videoCorePoolSize;
    }

    public int getVideoMaxPoolSize() {
        return videoMaxPoolSize;
    }

    public int getVideoQueueCapacity() {
        return videoQueueCapacity;
    }

    public double getVideoThumbnailTimeOffset() {
        return videoThumbnailTimeOffset;
    }

    public double getVideoThumbnailDuration() {
        return videoThumbnailDuration;
    }
}