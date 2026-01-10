package com.latte.album.dto;

import java.time.LocalDateTime;

public class MediaFileDTO {
    private String id;
    private String fileName;
    private String fileType;
    private String mimeType;
    private Long fileSize;
    private Integer width;
    private Integer height;
    private LocalDateTime exifTimestamp;
    private String exifTimezoneOffset;
    private LocalDateTime createTime;
    private LocalDateTime modifyTime;
    private String cameraMake;
    private String cameraModel;
    private String exposureTime;
    private String aperture;
    private Integer iso;
    private String focalLength;
    private Double duration;
    private String videoCodec;

    // Getters and Setters
    public String getId() {
        return id;
    }

    public void setId(String id) {
        this.id = id;
    }

    public String getFileName() {
        return fileName;
    }

    public void setFileName(String fileName) {
        this.fileName = fileName;
    }

    public String getFileType() {
        return fileType;
    }

    public void setFileType(String fileType) {
        this.fileType = fileType;
    }

    public String getMimeType() {
        return mimeType;
    }

    public void setMimeType(String mimeType) {
        this.mimeType = mimeType;
    }

    public Long getFileSize() {
        return fileSize;
    }

    public void setFileSize(Long fileSize) {
        this.fileSize = fileSize;
    }

    public Integer getWidth() {
        return width;
    }

    public void setWidth(Integer width) {
        this.width = width;
    }

    public Integer getHeight() {
        return height;
    }

    public void setHeight(Integer height) {
        this.height = height;
    }

    public LocalDateTime getExifTimestamp() {
        return exifTimestamp;
    }

    public void setExifTimestamp(LocalDateTime exifTimestamp) {
        this.exifTimestamp = exifTimestamp;
    }

    public String getExifTimezoneOffset() {
        return exifTimezoneOffset;
    }

    public void setExifTimezoneOffset(String exifTimezoneOffset) {
        this.exifTimezoneOffset = exifTimezoneOffset;
    }

    public LocalDateTime getCreateTime() {
        return createTime;
    }

    public void setCreateTime(LocalDateTime createTime) {
        this.createTime = createTime;
    }

    public LocalDateTime getModifyTime() {
        return modifyTime;
    }

    public void setModifyTime(LocalDateTime modifyTime) {
        this.modifyTime = modifyTime;
    }

    public String getCameraMake() {
        return cameraMake;
    }

    public void setCameraMake(String cameraMake) {
        this.cameraMake = cameraMake;
    }

    public String getCameraModel() {
        return cameraModel;
    }

    public void setCameraModel(String cameraModel) {
        this.cameraModel = cameraModel;
    }

    public String getExposureTime() {
        return exposureTime;
    }

    public void setExposureTime(String exposureTime) {
        this.exposureTime = exposureTime;
    }

    public String getAperture() {
        return aperture;
    }

    public void setAperture(String aperture) {
        this.aperture = aperture;
    }

    public Integer getIso() {
        return iso;
    }

    public void setIso(Integer iso) {
        this.iso = iso;
    }

    public String getFocalLength() {
        return focalLength;
    }

    public void setFocalLength(String focalLength) {
        this.focalLength = focalLength;
    }

    public Double getDuration() {
        return duration;
    }

    public void setDuration(Double duration) {
        this.duration = duration;
    }

    public String getVideoCodec() {
        return videoCodec;
    }

    public void setVideoCodec(String videoCodec) {
        this.videoCodec = videoCodec;
    }
}