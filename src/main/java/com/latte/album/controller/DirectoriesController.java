package com.latte.album.controller;

import com.latte.album.model.Directory;
import com.latte.album.repository.DirectoryRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;

@RestController
@RequestMapping("/api/directories")
public class DirectoriesController {
    
    private final DirectoryRepository directoryRepository;
    
    @Autowired
    public DirectoriesController(DirectoryRepository directoryRepository) {
        this.directoryRepository = directoryRepository;
    }
    
    @GetMapping
    public List<Directory> getDirectoryTree() {
        // 获取目录结构
        return directoryRepository.findAll();
    }
}