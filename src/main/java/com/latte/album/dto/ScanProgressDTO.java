package com.latte.album.dto;

import java.time.LocalDateTime;

public class ScanProgressDTO {

    private boolean scanning;
    private String phase;           // 当前阶段: collecting, counting, processing, deleting, completed
    private String phaseMessage;    // 阶段描述信息
    private long totalFiles;        // 需处理文件总数
    private long successCount;      // 成功处理数
    private long failureCount;      // 失败数
    private String progressPercentage;
    private LocalDateTime startTime;
    private String status;
    private String message;
    // 各阶段文件数量
    private long filesToAdd;        // 新增文件数
    private long filesToUpdate;     // 修改文件数
    private long filesToDelete;     // 删除文件数

    public ScanProgressDTO() {}

    public ScanProgressDTO(boolean scanning, String status) {
        this.scanning = scanning;
        this.status = status;
    }

    // 静态工厂方法
    public static ScanProgressDTO started() {
        ScanProgressDTO dto = new ScanProgressDTO(true, "started");
        dto.setPhase("started");
        dto.setMessage("扫描已启动");
        return dto;
    }

    /**
     * 创建扫描开始消息，包含各阶段文件数量
     */
    public static ScanProgressDTO started(long filesToAdd, long filesToUpdate, long filesToDelete) {
        ScanProgressDTO dto = new ScanProgressDTO(true, "started");
        dto.setPhase("processing");
        dto.setPhaseMessage("准备开始处理文件...");
        dto.setFilesToAdd(filesToAdd);
        dto.setFilesToUpdate(filesToUpdate);
        dto.setFilesToDelete(filesToDelete);
        dto.setTotalFiles(filesToAdd + filesToUpdate);
        dto.setMessage("扫描已启动");
        return dto;
    }

    public static ScanProgressDTO progress(long totalFiles, long successCount, long failureCount, String progressPercentage, LocalDateTime startTime) {
        ScanProgressDTO dto = new ScanProgressDTO(true, "progress");
        dto.setTotalFiles(totalFiles);
        dto.setSuccessCount(successCount);
        dto.setFailureCount(failureCount);
        dto.setProgressPercentage(progressPercentage);
        dto.setStartTime(startTime);
        return dto;
    }

    /**
     * 创建阶段更新消息
     */
    public static ScanProgressDTO phaseUpdate(String phase, String phaseMessage, long totalFiles,
                                              long successCount, long failureCount,
                                              String progressPercentage, LocalDateTime startTime) {
        ScanProgressDTO dto = new ScanProgressDTO(true, "progress");
        dto.setPhase(phase);
        dto.setPhaseMessage(phaseMessage);
        dto.setTotalFiles(totalFiles);
        dto.setSuccessCount(successCount);
        dto.setFailureCount(failureCount);
        dto.setProgressPercentage(progressPercentage);
        dto.setStartTime(startTime);
        return dto;
    }

    /**
     * 创建阶段更新消息（带各阶段数量）
     */
    public static ScanProgressDTO phaseUpdate(String phase, String phaseMessage, long totalFiles,
                                              long successCount, long failureCount,
                                              String progressPercentage, LocalDateTime startTime,
                                              long filesToAdd, long filesToUpdate, long filesToDelete) {
        ScanProgressDTO dto = new ScanProgressDTO(true, "progress");
        dto.setPhase(phase);
        dto.setPhaseMessage(phaseMessage);
        dto.setTotalFiles(totalFiles);
        dto.setSuccessCount(successCount);
        dto.setFailureCount(failureCount);
        dto.setProgressPercentage(progressPercentage);
        dto.setStartTime(startTime);
        dto.setFilesToAdd(filesToAdd);
        dto.setFilesToUpdate(filesToUpdate);
        dto.setFilesToDelete(filesToDelete);
        return dto;
    }

    public static ScanProgressDTO completed(long totalFiles, long successCount, long failureCount) {
        ScanProgressDTO dto = new ScanProgressDTO(false, "completed");
        dto.setPhase("completed");
        dto.setTotalFiles(totalFiles);
        dto.setSuccessCount(successCount);
        dto.setFailureCount(failureCount);
        dto.setProgressPercentage("100.00");
        dto.setMessage("扫描完成");
        return dto;
    }

    public static ScanProgressDTO error(String message) {
        ScanProgressDTO dto = new ScanProgressDTO(false, "error");
        dto.setPhase("error");
        dto.setMessage(message);
        return dto;
    }

    public static ScanProgressDTO cancelled() {
        ScanProgressDTO dto = new ScanProgressDTO(false, "cancelled");
        dto.setPhase("cancelled");
        dto.setMessage("扫描已取消");
        return dto;
    }

    // Getters and Setters
    public boolean isScanning() {
        return scanning;
    }

    public void setScanning(boolean scanning) {
        this.scanning = scanning;
    }

    public long getTotalFiles() {
        return totalFiles;
    }

    public void setTotalFiles(long totalFiles) {
        this.totalFiles = totalFiles;
    }

    public long getSuccessCount() {
        return successCount;
    }

    public void setSuccessCount(long successCount) {
        this.successCount = successCount;
    }

    public long getFailureCount() {
        return failureCount;
    }

    public void setFailureCount(long failureCount) {
        this.failureCount = failureCount;
    }

    public String getProgressPercentage() {
        return progressPercentage;
    }

    public void setProgressPercentage(String progressPercentage) {
        this.progressPercentage = progressPercentage;
    }

    public LocalDateTime getStartTime() {
        return startTime;
    }

    public void setStartTime(LocalDateTime startTime) {
        this.startTime = startTime;
    }

    public String getStatus() {
        return status;
    }

    public void setStatus(String status) {
        this.status = status;
    }

    public String getMessage() {
        return message;
    }

    public void setMessage(String message) {
        this.message = message;
    }

    public String getPhase() {
        return phase;
    }

    public void setPhase(String phase) {
        this.phase = phase;
    }

    public String getPhaseMessage() {
        return phaseMessage;
    }

    public void setPhaseMessage(String phaseMessage) {
        this.phaseMessage = phaseMessage;
    }

    public long getFilesToAdd() {
        return filesToAdd;
    }

    public void setFilesToAdd(long filesToAdd) {
        this.filesToAdd = filesToAdd;
    }

    public long getFilesToUpdate() {
        return filesToUpdate;
    }

    public void setFilesToUpdate(long filesToUpdate) {
        this.filesToUpdate = filesToUpdate;
    }

    public long getFilesToDelete() {
        return filesToDelete;
    }

    public void setFilesToDelete(long filesToDelete) {
        this.filesToDelete = filesToDelete;
    }
}
