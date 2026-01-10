package com.latte.album.controller;

import com.latte.album.dto.DateDTO;
import com.latte.album.dto.MediaFileDTO;
import com.latte.album.mapper.MediaFileMapper;
import com.latte.album.model.MediaFile;
import com.latte.album.service.HeifProcessorService;
import com.latte.album.service.MediaFileService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.data.domain.PageRequest;
import org.springframework.data.domain.Pageable;
import org.springframework.data.domain.Sort;
import org.springframework.http.HttpHeaders;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.stream.Collectors;

@RestController
@RequestMapping("/api/files")
public class FilesController {
    
    private static final Logger logger = LoggerFactory.getLogger(FilesController.class);
    
    private final MediaFileService mediaFileService;
    private final MediaFileMapper mediaFileMapper;
    private final HeifProcessorService heifProcessorService;
    
    @Autowired
    public FilesController(MediaFileService mediaFileService, 
                       MediaFileMapper mediaFileMapper,
                       HeifProcessorService heifProcessorService) {
        this.mediaFileService = mediaFileService;
        this.mediaFileMapper = mediaFileMapper;
        this.heifProcessorService = heifProcessorService;
    }
    
    @GetMapping
    public Page<MediaFileDTO> getFiles(
            @RequestParam(required = false) String path,
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "50") int size,
            @RequestParam(defaultValue = "exifTimestamp") String sortBy,
            @RequestParam(defaultValue = "desc") String order,
            @RequestParam(defaultValue = "all") String filterType,
            @RequestParam(required = false) String cameraModel,
            @RequestParam(required = false) String date) {
        // 获取文件列表（支持分页、排序、过滤、日期筛选）
        Sort.Direction sortDirection = "asc".equalsIgnoreCase(order) ? Sort.Direction.ASC : Sort.Direction.DESC;
        boolean descending = sortDirection == Sort.Direction.DESC;
        
        // 优化时间排序机制：优先使用EXIF拍摄时间，不存在时使用创建时间
        if ("exifTimestamp".equals(sortBy)) {
            // 当排序字段为exifTimestamp时，使用复合排序查询
            // 这样可以确保有EXIF时间和没有EXIF时间的照片混合在一起，而不是分成两组
            logger.info("使用EXIF拍摄时间排序，降级使用创建时间，排序方向: {}, 日期筛选: {}", sortDirection, date);
            Page<MediaFile> mediaFiles = mediaFileService.getFilesWithEffectiveSort(page, size, filterType, cameraModel, date, descending);
            
            List<MediaFileDTO> dtos = mediaFiles.getContent().stream()
                    .map(mediaFileMapper::toDTO)
                    .collect(Collectors.toList());
            
            return new PageImpl<>(dtos, mediaFiles.getPageable(), mediaFiles.getTotalElements());
        } else {
            // 其他排序字段保持原有逻辑
            Sort sort = Sort.by(sortDirection, sortBy);
            logger.debug("使用非时间字段排序: {}, 日期筛选: {}", sortBy, date);
            
            Pageable pageable = PageRequest.of(page, size, sort);
            Page<MediaFile> mediaFiles = mediaFileService.getFiles(pageable, filterType, cameraModel);
            
            List<MediaFileDTO> dtos = mediaFiles.getContent().stream()
                    .map(mediaFileMapper::toDTO)
                    .collect(Collectors.toList());
            
            return new PageImpl<>(dtos, pageable, mediaFiles.getTotalElements());
        }
    }
    
    @GetMapping("/{id}")
    public ResponseEntity<MediaFileDTO> getFileDetail(@PathVariable String id) {
        // 获取文件详情（含EXIF）
        return mediaFileService.getFileById(id)
                .map(mediaFile -> ResponseEntity.ok(mediaFileMapper.toDTO(mediaFile)))
                .orElse(ResponseEntity.notFound().build());
    }

    @GetMapping("/{id}/thumbnail")
    public ResponseEntity<byte[]> getThumbnail(@PathVariable String id,
                                              @RequestParam(defaultValue = "small") String size) {
        byte[] thumbnail = mediaFileService.getThumbnail(id, size);
        if (thumbnail.length == 0) {
            return ResponseEntity.notFound().build();
        }

        HttpHeaders headers = new HttpHeaders();
        headers.setContentType(MediaType.IMAGE_JPEG);
        headers.setContentLength(thumbnail.length);

        return new ResponseEntity<>(thumbnail, headers, HttpStatus.OK);
    }

    @GetMapping("/{id}/original")
    public ResponseEntity<byte[]> getOriginalFile(@PathVariable String id) {
        // 获取原始文件（用于下载，不转换）
        Optional<MediaFile> mediaFileOptional = mediaFileService.getFileById(id);
        if (!mediaFileOptional.isPresent()) {
            return ResponseEntity.notFound().build();
        }
        
        MediaFile mediaFile = mediaFileOptional.get();
        try {
            File file = new File(mediaFile.getFilePath());
            if (file.exists()) {
                byte[] fileData = Files.readAllBytes(file.toPath());
                HttpHeaders headers = new HttpHeaders();
                headers.setContentType(MediaType.parseMediaType(mediaFile.getMimeType()));
                headers.setContentLength(fileData.length);
                return new ResponseEntity<>(fileData, headers, HttpStatus.OK);
            } else {
                return ResponseEntity.notFound().build();
            }
        } catch (IOException e) {
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).build();
        }
    }
    
    @PutMapping("/{id}")
    public ResponseEntity<MediaFileDTO> updateFile(@PathVariable String id, @RequestBody MediaFileDTO fileDto) {
        // 更新文件信息（EXIF编辑）
        return ResponseEntity.ok(fileDto);
    }

    @GetMapping("/dates")
    public List<DateDTO> getDates(
            @RequestParam(defaultValue = "exifTimestamp") String sortBy,
            @RequestParam(defaultValue = "all") String filterType,
            @RequestParam(required = false) String cameraModel) {
        // 获取包含照片的日期列表
        logger.info("获取日期列表，排序方式: {}, 文件类型: {}, 相机型号: {}", sortBy, filterType, cameraModel);
        return mediaFileService.getDatesWithFilters(sortBy, filterType, cameraModel);
    }
}