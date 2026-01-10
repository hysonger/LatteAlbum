package com.latte.album.controller;

import com.latte.album.service.FileScannerService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.*;

import java.util.HashMap;
import java.util.Map;

@RestController
@RequestMapping("/api/system")
public class SystemController {
    
    private final FileScannerService fileScannerService;
    
    @Autowired
    public SystemController(FileScannerService fileScannerService) {
        this.fileScannerService = fileScannerService;
    }
    
    @PostMapping("/rescan")
    public Map<String, Object> rescan() {
        fileScannerService.scanDirectory();
        Map<String, Object> result = new HashMap<>();
        result.put("status", "started");
        result.put("message", "扫描已启动");
        return result;
    }
    
    @GetMapping("/scan/progress")
    public Map<String, Object> getScanProgress() {
        Map<String, Object> result = new HashMap<>();
        
        if (!fileScannerService.isScanning()) {
            result.put("scanning", false);
            result.put("message", "当前没有进行中的扫描");
        } else {
            FileScannerService.ScanProgress progress = fileScannerService.getScanProgress();
            result.put("scanning", true);
            result.put("totalFiles", progress.getTotalFiles());
            result.put("successCount", progress.getSuccessCount());
            result.put("failureCount", progress.getFailureCount());
            result.put("progressPercentage", String.format("%.2f", progress.getProgressPercentage()));
            result.put("startTime", progress.getStartTime());
        }
        
        return result;
    }
    
    @PostMapping("/scan/cancel")
    public Map<String, Object> cancelScan() {
        if (!fileScannerService.isScanning()) {
            Map<String, Object> result = new HashMap<>();
            result.put("status", "not_scanning");
            result.put("message", "当前没有进行中的扫描");
            return result;
        }
        
        fileScannerService.cancelScan();
        Map<String, Object> result = new HashMap<>();
        result.put("status", "cancelled");
        result.put("message", "已请求取消扫描");
        return result;
    }
    
    @GetMapping("/status")
    public Map<String, Object> getStatus() {
        Map<String, Object> status = new HashMap<>();
        status.put("status", "running");
        status.put("timestamp", System.currentTimeMillis());
        status.put("scanning", fileScannerService.isScanning());
        return status;
    }
}