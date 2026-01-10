package com.latte.album.util;

import com.latte.album.model.MediaFile;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.time.LocalDateTime;
import java.time.Year;

/**
 * 时间工具类，用于处理媒体文件的时间验证和排序逻辑
 * 
 * 功能说明：
 * 1. 验证EXIF拍摄时间的有效性（格式、范围等）
 * 2. 验证创建时间的有效性
 * 3. 提供有效的排序时间（优先使用EXIF时间，不存在时使用创建时间）
 * 
 * 优先级原则：拍摄时间优先，创建时间兜底
 */
public class TimeUtils {
    
    private static final Logger logger = LoggerFactory.getLogger(TimeUtils.class);
    
    /**
     * 最小有效年份：1900年（摄影术诞生之前）
     * 最大有效年份：当前年份+1年（预留未来空间）
     */
    private static final int MIN_VALID_YEAR = 1900;
    private static final int MAX_VALID_YEAR = Year.now().getValue() + 1;
    
    /**
     * 验证EXIF拍摄时间是否有效
     * 
     * 验证规则：
     * 1. 时间不能为null
     * 2. 年份必须在合理范围内（1900-当前年份+1）
     * 3. 时间不能是未来时间（可选，根据业务需求）
     * 
     * @param exifTimestamp EXIF拍摄时间
     * @return true表示有效，false表示无效
     */
    public static boolean isValidExifTimestamp(LocalDateTime exifTimestamp) {
        if (exifTimestamp == null) {
            return false;
        }
        
        int year = exifTimestamp.getYear();
        
        // 验证年份范围
        if (year < MIN_VALID_YEAR || year > MAX_VALID_YEAR) {
            logger.debug("EXIF时间年份超出有效范围: {}", exifTimestamp);
            return false;
        }
        
        // 可选：验证是否为未来时间（根据业务需求决定是否允许）
        // if (exifTimestamp.isAfter(LocalDateTime.now())) {
        //     logger.debug("EXIF时间为未来时间: {}", exifTimestamp);
        //     return false;
        // }
        
        return true;
    }
    
    /**
     * 验证创建时间是否有效
     * 
     * 验证规则：
     * 1. 时间不能为null
     * 2. 年份必须在合理范围内（1900-当前年份+1）
     * 3. 时间不能是未来时间
     * 
     * @param createTime 创建时间
     * @return true表示有效，false表示无效
     */
    public static boolean isValidCreateTime(LocalDateTime createTime) {
        if (createTime == null) {
            return false;
        }
        
        int year = createTime.getYear();
        
        // 验证年份范围
        if (year < MIN_VALID_YEAR || year > MAX_VALID_YEAR) {
            logger.debug("创建时间年份超出有效范围: {}", createTime);
            return false;
        }
        
        // 验证是否为未来时间
        if (createTime.isAfter(LocalDateTime.now())) {
            logger.debug("创建时间为未来时间: {}", createTime);
            return false;
        }
        
        return true;
    }
    
    /**
     * 获取有效的排序时间
     * 
     * 优先级规则：
     * 1. 首选检查EXIF拍摄时间，如果有效则使用
     * 2. 如果EXIF时间无效或不存在，降级使用创建时间
     * 3. 如果创建时间也无效，返回null（需要调用方处理）
     * 
     * 切换条件：
     * - EXIF时间存在且有效 → 使用EXIF时间
     * - EXIF时间不存在或无效 → 检查创建时间
     * - 创建时间存在且有效 → 使用创建时间
     * - 创建时间不存在或无效 → 返回null
     * 
     * @param mediaFile 媒体文件对象
     * @return 有效的排序时间，如果都无效则返回null
     */
    public static LocalDateTime getEffectiveSortTime(MediaFile mediaFile) {
        if (mediaFile == null) {
            logger.warn("媒体文件对象为null，无法获取排序时间");
            return null;
        }
        
        // 优先级1：检查EXIF拍摄时间
        LocalDateTime exifTimestamp = mediaFile.getExifTimestamp();
        if (exifTimestamp != null && isValidExifTimestamp(exifTimestamp)) {
            logger.debug("使用有效的EXIF拍摄时间作为排序依据: {} (文件: {})", 
                exifTimestamp, mediaFile.getFileName());
            return exifTimestamp;
        }
        
        // 优先级2：降级到创建时间
        LocalDateTime createTime = mediaFile.getCreateTime();
        if (createTime != null && isValidCreateTime(createTime)) {
            logger.debug("EXIF时间无效或不存在，使用创建时间作为排序依据: {} (文件: {})", 
                createTime, mediaFile.getFileName());
            return createTime;
        }
        
        // 兜底情况：两种时间都无效
        logger.warn("媒体文件的EXIF时间和创建时间都无效或不存在: {} (ID: {})", 
                mediaFile.getFileName(), mediaFile.getId());
        return null;
    }
    
    /**
     * 判断两个时间是否相等（用于去重和比较）
     * 
     * @param time1 第一个时间
     * @param time2 第二个时间
     * @return true表示相等，false表示不相等
     */
    public static boolean isTimeEqual(LocalDateTime time1, LocalDateTime time2) {
        if (time1 == null && time2 == null) {
            return true;
        }
        if (time1 == null || time2 == null) {
            return false;
        }
        return time1.equals(time2);
    }
    
    /**
     * 比较两个时间的先后顺序
     * 
     * @param time1 第一个时间
     * @param time2 第二个时间
     * @return 负数表示time1在time1之前，0表示相等，正数表示time1在time1之后
     */
    public static int compareTime(LocalDateTime time1, LocalDateTime time2) {
        if (time1 == null && time2 == null) {
            return 0;
        }
        if (time1 == null) {
            return -1; // time2更早
        }
        if (time2 == null) {
            return 1; // time1更早
        }
        return time1.compareTo(time2);
    }
}