package com.latte.album.config;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.scheduling.annotation.EnableAsync;
import org.springframework.scheduling.concurrent.ThreadPoolTaskExecutor;

import java.util.concurrent.Executor;

@Configuration
@EnableAsync
public class AsyncConfig {
    
    private final ApplicationConfig applicationConfig;
    
    @Autowired
    public AsyncConfig(ApplicationConfig applicationConfig) {
        this.applicationConfig = applicationConfig;
    }
    
    @Bean(name = "thumbnailGenerationExecutor")
    public Executor thumbnailGenerationExecutor() {
        ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
        int corePoolSize = applicationConfig.getThumbnailCorePoolSize() > 0 
            ? applicationConfig.getThumbnailCorePoolSize() 
            : Runtime.getRuntime().availableProcessors();
        int maxPoolSize = applicationConfig.getThumbnailMaxPoolSize() > 0 
            ? applicationConfig.getThumbnailMaxPoolSize() 
            : corePoolSize * 2;
        int queueCapacity = applicationConfig.getThumbnailQueueCapacity();
        
        executor.setCorePoolSize(corePoolSize);
        executor.setMaxPoolSize(maxPoolSize);
        executor.setQueueCapacity(queueCapacity);
        executor.setThreadNamePrefix("thumbnail-generator-");
        executor.setWaitForTasksToCompleteOnShutdown(true);
        executor.setAwaitTerminationSeconds(60);
        executor.setRejectedExecutionHandler(new java.util.concurrent.ThreadPoolExecutor.CallerRunsPolicy());
        executor.initialize();
        return executor;
    }
    
    @Bean(name = "heifConversionExecutor")
    public Executor heifConversionExecutor() {
        ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
        int corePoolSize = applicationConfig.getHeifCorePoolSize() > 0 
            ? applicationConfig.getHeifCorePoolSize() 
            : Math.max(4, Runtime.getRuntime().availableProcessors());
        int maxPoolSize = applicationConfig.getHeifMaxPoolSize() > 0 
            ? applicationConfig.getHeifMaxPoolSize() 
            : corePoolSize * 3;
        int queueCapacity = applicationConfig.getHeifQueueCapacity();
        
        executor.setCorePoolSize(corePoolSize);
        executor.setMaxPoolSize(maxPoolSize);
        executor.setQueueCapacity(queueCapacity);
        executor.setThreadNamePrefix("heif-converter-");
        executor.setWaitForTasksToCompleteOnShutdown(true);
        executor.setAwaitTerminationSeconds(120);
        executor.setRejectedExecutionHandler(new java.util.concurrent.ThreadPoolExecutor.CallerRunsPolicy());
        executor.initialize();
        return executor;
    }
    
    @Bean(name = "fileScanExecutor")
    public Executor fileScanExecutor() {
        ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
        int corePoolSize = Runtime.getRuntime().availableProcessors();
        executor.setCorePoolSize(corePoolSize);
        executor.setMaxPoolSize(corePoolSize * 2);
        executor.setQueueCapacity(500);
        executor.setThreadNamePrefix("file-scanner-");
        executor.setWaitForTasksToCompleteOnShutdown(true);
        executor.setAwaitTerminationSeconds(120);
        executor.setRejectedExecutionHandler(new java.util.concurrent.ThreadPoolExecutor.CallerRunsPolicy());
        executor.initialize();
        return executor;
    }
    
    @Bean(name = "videoProcessingExecutor")
    public Executor videoProcessingExecutor() {
        ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
        int corePoolSize = applicationConfig.getVideoCorePoolSize() > 0 
            ? applicationConfig.getVideoCorePoolSize() 
            : Runtime.getRuntime().availableProcessors();
        int maxPoolSize = applicationConfig.getVideoMaxPoolSize() > 0 
            ? applicationConfig.getVideoMaxPoolSize() 
            : corePoolSize * 2;
        int queueCapacity = applicationConfig.getVideoQueueCapacity();
        
        executor.setCorePoolSize(corePoolSize);
        executor.setMaxPoolSize(maxPoolSize);
        executor.setQueueCapacity(queueCapacity);
        executor.setThreadNamePrefix("video-processor-");
        executor.setWaitForTasksToCompleteOnShutdown(true);
        executor.setAwaitTerminationSeconds(300);
        executor.setRejectedExecutionHandler(new java.util.concurrent.ThreadPoolExecutor.CallerRunsPolicy());
        executor.initialize();
        return executor;
    }
}