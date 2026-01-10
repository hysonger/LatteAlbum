package com.latte.album.dto;

import java.time.LocalDateTime;

public class ScanProgressDTO {

    private boolean scanning;
    private long totalFiles;
    private long successCount;
    private long failureCount;
    private String progressPercentage;
    private LocalDateTime startTime;
    private String status; // started, progress, completed, error, cancelled
    private String message;

    public ScanProgressDTO() {}

    public ScanProgressDTO(boolean scanning, String status) {
        this.scanning = scanning;
        this.status = status;
    }

    // 静态工厂方法
    public static ScanProgressDTO started() {
        ScanProgressDTO dto = new ScanProgressDTO(true, "started");
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

    public static ScanProgressDTO completed(long totalFiles, long successCount, long failureCount) {
        ScanProgressDTO dto = new ScanProgressDTO(false, "completed");
        dto.setTotalFiles(totalFiles);
        dto.setSuccessCount(successCount);
        dto.setFailureCount(failureCount);
        dto.setProgressPercentage("100.00");
        dto.setMessage("扫描完成");
        return dto;
    }

    public static ScanProgressDTO error(String message) {
        ScanProgressDTO dto = new ScanProgressDTO(false, "error");
        dto.setMessage(message);
        return dto;
    }

    public static ScanProgressDTO cancelled() {
        ScanProgressDTO dto = new ScanProgressDTO(false, "cancelled");
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
}
