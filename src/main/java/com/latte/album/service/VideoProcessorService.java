package com.latte.album.service;

import com.latte.album.config.ApplicationConfig;
import com.latte.album.model.MediaFile;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;
import ws.schild.jave.Encoder;
import ws.schild.jave.MultimediaObject;
import ws.schild.jave.info.MultimediaInfo;
import ws.schild.jave.info.VideoInfo;

import java.io.File;
import java.nio.file.Files;
import java.util.concurrent.CompletableFuture;

@Service
public class VideoProcessorService {
    
    private static final Logger logger = LoggerFactory.getLogger(VideoProcessorService.class);
    
    private final ApplicationConfig applicationConfig;
    private volatile boolean videoProcessingAvailable = false;
    
    public VideoProcessorService(ApplicationConfig applicationConfig) {
        this.applicationConfig = applicationConfig;
        checkVideoProcessingAvailability();
    }
    
    private void checkVideoProcessingAvailability() {
        try {
            Encoder encoder = new Encoder();
            videoProcessingAvailable = true;
            logger.info("视频处理功能可用");
        } catch (Exception e) {
            videoProcessingAvailable = false;
            logger.warn("视频处理功能不可用: {}", e.getMessage());
        }
    }
    
    public boolean isVideoProcessingAvailable() {
        return videoProcessingAvailable;
    }
    
    public void extractVideoMetadata(File videoFile, MediaFile mediaFile) {
        if (!videoProcessingAvailable) {
            logger.warn("视频处理功能不可用，跳过元数据提取: {}", videoFile.getName());
            return;
        }
        
        try {
            MultimediaObject multimediaObject = new MultimediaObject(videoFile);
            MultimediaInfo info = multimediaObject.getInfo();
            
            VideoInfo videoInfo = info.getVideo();
            if (videoInfo != null) {
                mediaFile.setWidth(videoInfo.getSize().getWidth());
                mediaFile.setHeight(videoInfo.getSize().getHeight());
                mediaFile.setVideoCodec(videoInfo.getDecoder());
            }
            
            long durationMillis = info.getDuration();
            mediaFile.setDuration(durationMillis / 1000.0);
            
            logger.debug("成功提取视频元数据: {} (时长: {}s, 分辨率: {}x{}, 编码: {})", 
                videoFile.getName(), 
                mediaFile.getDuration(),
                mediaFile.getWidth(),
                mediaFile.getHeight(),
                mediaFile.getVideoCodec());
        } catch (Exception e) {
            logger.error("提取视频元数据失败: {}", videoFile.getName(), e);
        }
    }
    
    public byte[] generateThumbnail(File videoFile, int width) {
        if (!videoProcessingAvailable) {
            logger.warn("视频处理功能不可用，无法生成缩略图: {}", videoFile.getName());
            return new byte[0];
        }
        
        if (videoFile == null || !videoFile.exists()) {
            logger.warn("视频文件不存在: {}", videoFile);
            return new byte[0];
        }
        
        try {
            File tempFile = File.createTempFile("video_thumb_", ".jpg");
            tempFile.deleteOnExit();
            
            Encoder encoder = new Encoder();
            
            MultimediaObject multimediaObject = new MultimediaObject(videoFile);
            MultimediaInfo info = multimediaObject.getInfo();
            
            VideoInfo videoInfo = info.getVideo();
            if (videoInfo == null) {
                logger.warn("无法获取视频信息: {}", videoFile.getName());
                return new byte[0];
            }
            
            int originalWidth = videoInfo.getSize().getWidth();
            int originalHeight = videoInfo.getSize().getHeight();
            
            int targetWidth = width > 0 ? width : originalWidth;
            int targetHeight = (int) ((double) originalHeight * targetWidth / originalWidth);
            
            ws.schild.jave.encode.VideoAttributes videoAttributes = new ws.schild.jave.encode.VideoAttributes();
            videoAttributes.setCodec("mjpeg");
            videoAttributes.setSize(new ws.schild.jave.info.VideoSize(targetWidth, targetHeight));
            videoAttributes.setFrameRate(1);
            
            ws.schild.jave.encode.AudioAttributes audioAttributes = new ws.schild.jave.encode.AudioAttributes();
            audioAttributes.setCodec("none");
            
            ws.schild.jave.encode.EncodingAttributes attrs = new ws.schild.jave.encode.EncodingAttributes();
            attrs.setOffset((float) applicationConfig.getVideoThumbnailTimeOffset());
            attrs.setDuration((float) applicationConfig.getVideoThumbnailDuration());
            attrs.setAudioAttributes(audioAttributes);
            attrs.setVideoAttributes(videoAttributes);
            
            encoder.encode(multimediaObject, tempFile, attrs);
            
            byte[] thumbnailBytes = Files.readAllBytes(tempFile.toPath());
            
            if (!tempFile.delete()) {
                tempFile.deleteOnExit();
            }
            
            logger.debug("成功生成视频缩略图: {} (大小: {} bytes)", videoFile.getName(), thumbnailBytes.length);
            return thumbnailBytes;
            
        } catch (Exception e) {
            logger.error("生成视频缩略图失败: {}", videoFile.getName(), e);
            return new byte[0];
        }
    }
    
    @Async("videoProcessingExecutor")
    public CompletableFuture<byte[]> generateThumbnailAsync(File videoFile, int width) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                logger.debug("开始异步生成视频缩略图: {}, width: {}", videoFile.getName(), width);
                byte[] result = generateThumbnail(videoFile, width);
                logger.debug("异步生成视频缩略图完成: {}, 大小: {} bytes", 
                    videoFile.getName(), result.length);
                return result;
            } catch (Exception e) {
                logger.error("异步生成视频缩略图失败: {}", videoFile.getName(), e);
                return new byte[0];
            }
        });
    }
    
    public String getVideoCodec(File videoFile) {
        if (!videoProcessingAvailable) {
            return null;
        }
        
        try {
            MultimediaObject multimediaObject = new MultimediaObject(videoFile);
            MultimediaInfo info = multimediaObject.getInfo();
            VideoInfo videoInfo = info.getVideo();
            
            if (videoInfo != null) {
                return videoInfo.getDecoder();
            }
        } catch (Exception e) {
            logger.error("获取视频编码格式失败: {}", videoFile.getName(), e);
        }
        
        return null;
    }
    
    public boolean isH264Video(File videoFile) {
        String codec = getVideoCodec(videoFile);
        return codec != null && (codec.toLowerCase().contains("h264") || 
                                  codec.toLowerCase().contains("avc"));
    }
    
    public boolean isH265Video(File videoFile) {
        String codec = getVideoCodec(videoFile);
        return codec != null && (codec.toLowerCase().contains("h265") || 
                                  codec.toLowerCase().contains("hevc"));
    }
}