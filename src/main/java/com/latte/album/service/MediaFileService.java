package com.latte.album.service;

import com.latte.album.dto.DateDTO;
import com.latte.album.model.MediaFile;
import com.latte.album.repository.MediaFileRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.data.domain.PageRequest;
import org.springframework.data.domain.Pageable;
import org.springframework.stereotype.Service;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

@Service
public class MediaFileService {
    
    private final MediaFileRepository mediaFileRepository;
    private final CacheService cacheService;
    
    @Autowired
    public MediaFileService(MediaFileRepository mediaFileRepository, 
                           CacheService cacheService) {
        this.mediaFileRepository = mediaFileRepository;
        this.cacheService = cacheService;
    }
    
    public Page<MediaFile> getFiles(Pageable pageable, String filterType, String cameraModel) {
        // 根据过滤条件调用不同的查询方法
        if ("all".equals(filterType) && cameraModel == null) {
            return mediaFileRepository.findAll(pageable);
        } else if ("all".equals(filterType)) {
            return mediaFileRepository.findByCameraModel(cameraModel, pageable);
        } else if (cameraModel == null) {
            return mediaFileRepository.findByFileType(filterType, pageable);
        } else {
            return mediaFileRepository.findByCameraModelAndFileType(cameraModel, filterType, pageable);
        }
    }
    
    public Page<MediaFile> getFilesWithEffectiveSort(int page, int size, String filterType, String cameraModel, String date, boolean descending) {
        // 使用复合排序查询：优先使用EXIF时间，不存在时使用创建时间
        // 这样可以确保有EXIF时间和没有EXIF时间的照片混合在一起，而不是分成两组
        int offset = page * size;
        
        // 处理日期筛选参数 - 使用 UTC 时区避免时区转换问题
        Long startTimestamp = null;
        Long endTimestamp = null;
        if (date != null && !date.isEmpty()) {
            try {
                java.time.LocalDate localDate = java.time.LocalDate.parse(date);
                java.time.LocalDateTime startOfDay = java.time.LocalDateTime.of(localDate, java.time.LocalTime.MIN);
                java.time.LocalDateTime endOfDay = java.time.LocalDateTime.of(localDate, java.time.LocalTime.MAX);
                startTimestamp = startOfDay.atZone(java.time.ZoneOffset.UTC).toInstant().toEpochMilli();
                endTimestamp = endOfDay.atZone(java.time.ZoneOffset.UTC).toInstant().toEpochMilli();
            } catch (Exception e) {
                // 日期解析失败，忽略日期筛选
            }
        }
        
        long total = mediaFileRepository.countWithFilters(filterType, cameraModel, date, startTimestamp, endTimestamp);
        
        List<MediaFile> files;
        if (descending) {
            files = mediaFileRepository.findAllWithEffectiveSortDesc(filterType, cameraModel, date, startTimestamp, endTimestamp, size, offset);
        } else {
            files = mediaFileRepository.findAllWithEffectiveSortAsc(filterType, cameraModel, date, startTimestamp, endTimestamp, size, offset);
        }
        
        // 创建Pageable对象用于PageImpl
        Pageable pageable = PageRequest.of(page, size);
        
        // 创建Page对象
        return new PageImpl<>(files, pageable, total);
    }
    
    public Optional<MediaFile> getFileById(String id) {
        return mediaFileRepository.findById(id);
    }

    public byte[] getThumbnail(String id, String size) {
        return mediaFileRepository.findById(id)
                .map(mediaFile -> cacheService.getThumbnail(mediaFile, size))
                .orElse(new byte[0]);
    }

    public List<DateDTO> getDatesWithFilters(String sortBy, String filterType, String cameraModel) {
        List<Object[]> result;

        if ("exifTimestamp".equals(sortBy)) {
            result = mediaFileRepository.findDatesByExifTimestamp(filterType, cameraModel);
        } else if ("createTime".equals(sortBy)) {
            result = mediaFileRepository.findDatesByCreateTime(filterType, cameraModel);
        } else if ("modifyTime".equals(sortBy)) {
            result = mediaFileRepository.findDatesByModifyTime(filterType, cameraModel);
        } else {
            result = mediaFileRepository.findDatesByAnyTime(filterType, cameraModel);
        }

        List<DateDTO> dates = new ArrayList<>();
        for (Object[] row : result) {
            String date = (String) row[0];
            Number count = (Number) row[1];
            dates.add(new DateDTO(date, count.longValue()));
        }

        return dates;
    }
}