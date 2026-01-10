package com.latte.album.mapper;

import com.latte.album.dto.MediaFileDTO;
import com.latte.album.model.MediaFile;
import org.springframework.stereotype.Component;

@Component
public class MediaFileMapper {
    
    public MediaFileDTO toDTO(MediaFile mediaFile) {
        if (mediaFile == null) {
            return null;
        }
        
        MediaFileDTO dto = new MediaFileDTO();
        dto.setId(mediaFile.getId());
        dto.setFileName(mediaFile.getFileName());
        dto.setFileType(mediaFile.getFileType());
        dto.setMimeType(mediaFile.getMimeType());
        dto.setFileSize(mediaFile.getFileSize());
        dto.setWidth(mediaFile.getWidth());
        dto.setHeight(mediaFile.getHeight());
        dto.setExifTimestamp(mediaFile.getExifTimestamp());
        dto.setExifTimezoneOffset(mediaFile.getExifTimezoneOffset());
        dto.setCreateTime(mediaFile.getCreateTime());
        dto.setModifyTime(mediaFile.getModifyTime());
        dto.setCameraMake(mediaFile.getCameraMake());
        dto.setCameraModel(mediaFile.getCameraModel());
        dto.setExposureTime(mediaFile.getExposureTime());
        dto.setAperture(mediaFile.getAperture());
        dto.setIso(mediaFile.getIso());
        dto.setFocalLength(mediaFile.getFocalLength());
        dto.setDuration(mediaFile.getDuration());
        dto.setVideoCodec(mediaFile.getVideoCodec());
        
        return dto;
    }
}