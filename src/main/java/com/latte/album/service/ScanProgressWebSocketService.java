package com.latte.album.service;

import com.latte.album.dto.ScanProgressDTO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.messaging.simp.SimpMessagingTemplate;
import org.springframework.stereotype.Service;

import java.time.LocalDateTime;

@Service
public class ScanProgressWebSocketService {

    private static final Logger logger = LoggerFactory.getLogger(ScanProgressWebSocketService.class);
    private static final String SCAN_PROGRESS_TOPIC = "/topic/scan/progress";

    private final SimpMessagingTemplate messagingTemplate;

    public ScanProgressWebSocketService(SimpMessagingTemplate messagingTemplate) {
        this.messagingTemplate = messagingTemplate;
    }

    /**
     * 发送扫描启动消息（基础版本）
     */
    public void sendScanStarted() {
        ScanProgressDTO dto = ScanProgressDTO.started();
        messagingTemplate.convertAndSend(SCAN_PROGRESS_TOPIC, dto);
        logger.info("WebSocket: 发送扫描启动消息");
    }

    /**
     * 发送扫描启动消息（包含各阶段文件数量）
     */
    public void sendScanStarted(long filesToAdd, long filesToUpdate, long filesToDelete) {
        ScanProgressDTO dto = ScanProgressDTO.started(filesToAdd, filesToUpdate, filesToDelete);
        messagingTemplate.convertAndSend(SCAN_PROGRESS_TOPIC, dto);
        logger.info("WebSocket: 发送扫描启动消息 - 新增:{}, 修改:{}, 删除:{}", filesToAdd, filesToUpdate, filesToDelete);
    }

    /**
     * 发送阶段更新消息
     */
    public void sendPhaseUpdate(String phase, String phaseMessage, long totalFiles,
                                long successCount, long failureCount,
                                String progressPercentage, LocalDateTime startTime) {
        ScanProgressDTO dto = ScanProgressDTO.phaseUpdate(phase, phaseMessage, totalFiles,
            successCount, failureCount, progressPercentage, startTime);
        messagingTemplate.convertAndSend(SCAN_PROGRESS_TOPIC, dto);
        logger.debug("WebSocket: 发送阶段更新 - {}: {} ({})", phase, phaseMessage, progressPercentage);
    }

    /**
     * 发送阶段更新消息（包含各阶段数量）
     */
    public void sendPhaseUpdate(String phase, String phaseMessage, long totalFiles,
                                long successCount, long failureCount,
                                String progressPercentage, LocalDateTime startTime,
                                long filesToAdd, long filesToUpdate, long filesToDelete) {
        ScanProgressDTO dto = ScanProgressDTO.phaseUpdate(phase, phaseMessage, totalFiles,
            successCount, failureCount, progressPercentage, startTime,
            filesToAdd, filesToUpdate, filesToDelete);
        messagingTemplate.convertAndSend(SCAN_PROGRESS_TOPIC, dto);
        logger.debug("WebSocket: 发送阶段更新 - {}: {} ({})", phase, phaseMessage, progressPercentage);
    }

    /**
     * 发送扫描进度更新
     */
    public void sendProgressUpdate(long totalFiles, long successCount, long failureCount, String progressPercentage) {
        ScanProgressDTO dto = ScanProgressDTO.progress(totalFiles, successCount, failureCount, progressPercentage, null);
        messagingTemplate.convertAndSend(SCAN_PROGRESS_TOPIC, dto);
        logger.debug("WebSocket: 发送进度更新 {}/{} ({})", successCount + failureCount, totalFiles, progressPercentage);
    }

    /**
     * 发送扫描完成消息
     */
    public void sendScanCompleted(long totalFiles, long successCount, long failureCount) {
        ScanProgressDTO dto = ScanProgressDTO.completed(totalFiles, successCount, failureCount);
        messagingTemplate.convertAndSend(SCAN_PROGRESS_TOPIC, dto);
        logger.info("WebSocket: 发送扫描完成消息 - 总:{}, 成功:{}, 失败:{}", totalFiles, successCount, failureCount);
    }

    /**
     * 发送扫描错误消息
     */
    public void sendScanError(String errorMessage) {
        ScanProgressDTO dto = ScanProgressDTO.error(errorMessage);
        messagingTemplate.convertAndSend(SCAN_PROGRESS_TOPIC, dto);
        logger.error("WebSocket: 发送扫描错误消息 - {}", errorMessage);
    }

    /**
     * 发送扫描取消消息
     */
    public void sendScanCancelled() {
        ScanProgressDTO dto = ScanProgressDTO.cancelled();
        messagingTemplate.convertAndSend(SCAN_PROGRESS_TOPIC, dto);
        logger.info("WebSocket: 发送扫描取消消息");
    }
}
