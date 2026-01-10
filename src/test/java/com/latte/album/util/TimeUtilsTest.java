package com.latte.album.util;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import java.time.LocalDateTime;

import static org.junit.jupiter.api.Assertions.*;

@ExtendWith(MockitoExtension.class)
class TimeUtilsTest {
    
    /**
     * 测试TimeUtils.isValidExifTimestamp方法
     */
    @Test
    void testIsValidExifTimestamp_ValidTime() {
        LocalDateTime validTime = LocalDateTime.of(2023, 5, 15, 10, 30);
        assertTrue(TimeUtils.isValidExifTimestamp(validTime));
    }
    
    @Test
    void testIsValidExifTimestamp_NullTime() {
        assertFalse(TimeUtils.isValidExifTimestamp(null));
    }
    
    @Test
    void testIsValidExifTimestamp_TooEarlyYear() {
        LocalDateTime tooEarlyTime = LocalDateTime.of(1899, 12, 31, 1, 1);
        assertFalse(TimeUtils.isValidExifTimestamp(tooEarlyTime));
    }
    
    @Test
    void testIsValidExifTimestamp_FutureYear() {
        LocalDateTime futureTime = LocalDateTime.now().plusYears(2);
        assertFalse(TimeUtils.isValidExifTimestamp(futureTime));
    }
    
    @Test
    void testIsValidExifTimestamp_BoundaryYear() {
        LocalDateTime boundaryTime = LocalDateTime.of(1900, 1, 1, 0, 0);
        assertTrue(TimeUtils.isValidExifTimestamp(boundaryTime));
    }
    
    /**
     * 测试TimeUtils.isValidCreateTime方法
     */
    @Test
    void testIsValidCreateTime_ValidTime() {
        LocalDateTime validTime = LocalDateTime.of(2023, 5, 15, 10, 30);
        assertTrue(TimeUtils.isValidCreateTime(validTime));
    }
    
    @Test
    void testIsValidCreateTime_NullTime() {
        assertFalse(TimeUtils.isValidCreateTime(null));
    }
    
    @Test
    void testIsValidCreateTime_TooEarlyYear() {
        LocalDateTime tooEarlyTime = LocalDateTime.of(1899, 12, 31, 1, 1);
        assertFalse(TimeUtils.isValidCreateTime(tooEarlyTime));
    }
    
    @Test
    void testIsValidCreateTime_FutureYear() {
        LocalDateTime futureTime = LocalDateTime.now().plusYears(2);
        assertFalse(TimeUtils.isValidCreateTime(futureTime));
    }
    
    @Test
    void testIsValidCreateTime_BoundaryYear() {
        LocalDateTime boundaryTime = LocalDateTime.of(1900, 1, 1, 0, 0);
        assertTrue(TimeUtils.isValidCreateTime(boundaryTime));
    }
    
    /**
     * 测试TimeUtils.isTimeEqual方法
     */
    @Test
    void testIsTimeEqual_BothNull() {
        assertTrue(TimeUtils.isTimeEqual(null, null));
    }
    
    @Test
    void testIsTimeEqual_OneNull() {
        LocalDateTime time = LocalDateTime.of(2023, 5, 15, 10, 30);
        assertFalse(TimeUtils.isTimeEqual(null, time));
        assertFalse(TimeUtils.isTimeEqual(time, null));
    }
    
    @Test
    void testIsTimeEqual_BothValid() {
        LocalDateTime time1 = LocalDateTime.of(2023, 5, 15, 10, 30);
        LocalDateTime time2 = LocalDateTime.of(2023, 5, 15, 10, 30);
        assertTrue(TimeUtils.isTimeEqual(time1, time2));
    }
    
    @Test
    void testIsTimeEqual_DifferentTimes() {
        LocalDateTime time1 = LocalDateTime.of(2023, 5, 15, 10, 30);
        LocalDateTime time2 = LocalDateTime.of(2023, 5, 15, 10, 31);
        assertFalse(TimeUtils.isTimeEqual(time1, time2));
    }
    
    /**
     * 测试TimeUtils.compareTime方法
     */
    @Test
    void testCompareTime_BothNull() {
        assertEquals(0, TimeUtils.compareTime(null, null));
    }
    
    @Test
    void testCompareTime_FirstNull() {
        LocalDateTime time = LocalDateTime.of(2023, 5, 15, 10, 30);
        assertEquals(-1, TimeUtils.compareTime(null, time));
    }
    
    @Test
    void testCompareTime_SecondNull() {
        LocalDateTime time = LocalDateTime.of(2023, 5, 15, 10, 30);
        assertEquals(1, TimeUtils.compareTime(time, null));
    }
    
    @Test
    void testCompareTime_Earlier() {
        LocalDateTime time1 = LocalDateTime.of(2023, 5, 15, 10, 30);
        LocalDateTime time2 = LocalDateTime.of(2023, 5, 15, 10, 31);
        assertTrue(TimeUtils.compareTime(time1, time2) < 0);
    }
    
    @Test
    void testCompareTime_Later() {
        LocalDateTime time1 = LocalDateTime.of(2023, 5, 15, 10, 31);
        LocalDateTime time2 = LocalDateTime.of(2023, 5, 15, 10, 30);
        assertTrue(TimeUtils.compareTime(time1, time2) > 0);
    }
    
    @Test
    void testCompareTime_Equal() {
        LocalDateTime time1 = LocalDateTime.of(2023, 5, 15, 10, 30);
        LocalDateTime time2 = LocalDateTime.of(2023, 5, 15, 10, 30);
        assertEquals(0, TimeUtils.compareTime(time1, time2));
    }
    
    /**
     * 测试TimeUtils.getEffectiveSortTime方法
     */
    @Test
    void testGetEffectiveSortTime_ValidExifTime() {
        com.latte.album.model.MediaFile mediaFile = new com.latte.album.model.MediaFile();
        LocalDateTime validExifTime = LocalDateTime.of(2023, 5, 15, 10, 30);
        LocalDateTime validCreateTime = LocalDateTime.of(2023, 5, 15, 10, 30);
        
        mediaFile.setId(java.util.UUID.randomUUID().toString());
        mediaFile.setFileName("test.jpg");
        mediaFile.setExifTimestamp(validExifTime);
        mediaFile.setCreateTime(validCreateTime);

        LocalDateTime effectiveTime = TimeUtils.getEffectiveSortTime(mediaFile);
        assertEquals(validExifTime, effectiveTime);
    }

    @Test
    void testGetEffectiveSortTime_InvalidExifTime() {
        com.latte.album.model.MediaFile mediaFile = new com.latte.album.model.MediaFile();
        LocalDateTime invalidExifTime = LocalDateTime.of(1800, 1, 1, 1, 1);
        LocalDateTime validCreateTime = LocalDateTime.of(2023, 5, 15, 10, 30);

        mediaFile.setId(java.util.UUID.randomUUID().toString());
        mediaFile.setFileName("test.jpg");
        mediaFile.setExifTimestamp(invalidExifTime);
        mediaFile.setCreateTime(validCreateTime);

        LocalDateTime effectiveTime = TimeUtils.getEffectiveSortTime(mediaFile);
        assertEquals(validCreateTime, effectiveTime);
    }

    @Test
    void testGetEffectiveSortTime_NoExifTime() {
        com.latte.album.model.MediaFile mediaFile = new com.latte.album.model.MediaFile();
        LocalDateTime validCreateTime = LocalDateTime.of(2023, 5, 15, 10, 30);

        mediaFile.setId(java.util.UUID.randomUUID().toString());
        mediaFile.setFileName("test.jpg");
        mediaFile.setExifTimestamp(null);
        mediaFile.setCreateTime(validCreateTime);

        LocalDateTime effectiveTime = TimeUtils.getEffectiveSortTime(mediaFile);
        assertEquals(validCreateTime, effectiveTime);
    }

    @Test
    void testGetEffectiveSortTime_NoValidTime() {
        com.latte.album.model.MediaFile mediaFile = new com.latte.album.model.MediaFile();
        LocalDateTime invalidExifTime = LocalDateTime.of(1800, 1, 1, 1, 1);
        LocalDateTime invalidCreateTime = LocalDateTime.of(1800, 1, 1, 1, 1);

        mediaFile.setId(java.util.UUID.randomUUID().toString());
        mediaFile.setFileName("test.jpg");
        mediaFile.setExifTimestamp(invalidExifTime);
        mediaFile.setCreateTime(invalidCreateTime);

        LocalDateTime effectiveTime = TimeUtils.getEffectiveSortTime(mediaFile);
        assertNull(effectiveTime);
    }

    @Test
    void testGetEffectiveSortTime_NullFile() {
        LocalDateTime effectiveTime = TimeUtils.getEffectiveSortTime(null);
        assertNull(effectiveTime);
    }

    @Test
    void testGetEffectiveSortTime_ExifTimePriority() {
        com.latte.album.model.MediaFile mediaFile = new com.latte.album.model.MediaFile();
        LocalDateTime exifTime = LocalDateTime.of(2023, 5, 15, 10, 30);
        LocalDateTime createTime = LocalDateTime.of(2023, 5, 16, 11, 45);

        mediaFile.setId(java.util.UUID.randomUUID().toString());
        mediaFile.setFileName("test.jpg");
        mediaFile.setExifTimestamp(exifTime);
        mediaFile.setCreateTime(createTime);
        
        LocalDateTime effectiveTime = TimeUtils.getEffectiveSortTime(mediaFile);
        assertEquals(exifTime, effectiveTime);
        assertNotEquals(createTime, effectiveTime);
    }
}