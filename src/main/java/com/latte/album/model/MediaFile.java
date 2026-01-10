package com.latte.album.model;

import jakarta.persistence.*;
import java.time.LocalDateTime;
import java.util.UUID;

@Entity
@Table(name = "media_files")
public class MediaFile {
    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private String id;
    
    @Column(name = "file_path", nullable = false, unique = true)
    private String filePath;
    
    @Column(name = "file_name", nullable = false)
    private String fileName;
    
    @Column(name = "file_type", nullable = false)
    private String fileType; // 'image' or 'video'
    
    @Column(name = "mime_type")
    private String mimeType;
    
    @Column(name = "file_size")
    private Long fileSize;
    
    @Column(name = "width")
    private Integer width;
    
    @Column(name = "height")
    private Integer height;
    
    @Column(name = "exif_timestamp")
    private LocalDateTime exifTimestamp;

    @Column(name = "exif_timezone_offset", length = 7)
    private String exifTimezoneOffset;

    @Column(name = "create_time")
    private LocalDateTime createTime;

    @Column(name = "modify_time")
    private LocalDateTime modifyTime;
    
    @Column(name = "camera_make")
    private String cameraMake;
    
    @Column(name = "camera_model")
    private String cameraModel;
    
    @Column(name = "lens_model")
    private String lensModel;
    
    @Column(name = "exposure_time")
    private String exposureTime;
    
    @Column(name = "aperture")
    private String aperture;
    
    @Column(name = "iso")
    private Integer iso;
    
    @Column(name = "focal_length")
    private String focalLength;
    
    @Column(name = "duration")
    private Double duration;
    
    @Column(name = "video_codec")
    private String videoCodec;
    
    @Column(name = "thumbnail_generated")
    private Boolean thumbnailGenerated = false;
    
    @Column(name = "last_scanned")
    private LocalDateTime lastScanned;
    
    // Getters and setters
    public String getId() {
        return id;
    }

    public void setId(String id) {
        this.id = id;
    }
    
    public String getFilePath() {
        return filePath;
    }
    
    public void setFilePath(String filePath) {
        this.filePath = filePath;
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
    
    public String getLensModel() {
        return lensModel;
    }
    
    public void setLensModel(String lensModel) {
        this.lensModel = lensModel;
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
    
    public Boolean getThumbnailGenerated() {
        return thumbnailGenerated;
    }
    
    public void setThumbnailGenerated(Boolean thumbnailGenerated) {
        this.thumbnailGenerated = thumbnailGenerated;
    }
    
    public LocalDateTime getLastScanned() {
        return lastScanned;
    }
    
    public void setLastScanned(LocalDateTime lastScanned) {
        this.lastScanned = lastScanned;
    }
}