package com.latte.album.model;

import jakarta.persistence.*;
import java.time.LocalDateTime;

@Entity
@Table(name = "directories")
public class Directory {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;
    
    @Column(name = "path", nullable = false)
    private String path;
    
    @Column(name = "parent_id")
    private Long parentId;
    
    @Column(name = "file_count")
    private Integer fileCount = 0;
    
    @Column(name = "last_modified")
    private LocalDateTime lastModified;
    
    // Getters and setters
    public Long getId() {
        return id;
    }
    
    public void setId(Long id) {
        this.id = id;
    }
    
    public String getPath() {
        return path;
    }
    
    public void setPath(String path) {
        this.path = path;
    }
    
    public Long getParentId() {
        return parentId;
    }
    
    public void setParentId(Long parentId) {
        this.parentId = parentId;
    }
    
    public Integer getFileCount() {
        return fileCount;
    }
    
    public void setFileCount(Integer fileCount) {
        this.fileCount = fileCount;
    }
    
    public LocalDateTime getLastModified() {
        return lastModified;
    }
    
    public void setLastModified(LocalDateTime lastModified) {
        this.lastModified = lastModified;
    }
}