package com.latte.album.task;

import com.latte.album.config.ApplicationConfig;
import com.latte.album.service.FileScannerService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;

@Component
public class ScheduledTasks {
    
    private static final Logger logger = LoggerFactory.getLogger(ScheduledTasks.class);
    
    private final ApplicationConfig applicationConfig;
    private final FileScannerService fileScannerService;
    
    @Autowired
    public ScheduledTasks(ApplicationConfig applicationConfig, 
                          FileScannerService fileScannerService) {
        this.applicationConfig = applicationConfig;
        this.fileScannerService = fileScannerService;
    }
    
    /**
     * 定时扫描目录
     * 默认每天凌晨2点执行
     */
    @Scheduled(cron = "${album.scan.cron:0 0 2 * * ?}")
    public void scheduledScan() {
        if (applicationConfig.isScanEnabled()) {
            logger.info("开始执行定时扫描任务");
            fileScannerService.scanDirectory();
            logger.info("定时扫描任务执行完成");
        }
    }
}