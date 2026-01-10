package com.latte.album.repository;

import com.latte.album.model.MediaFile;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.domain.Sort;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.util.List;
import java.util.Optional;

@Repository
public interface MediaFileRepository extends JpaRepository<MediaFile, String> {
    List<MediaFile> findByFileType(String fileType);
    List<MediaFile> findByCameraModel(String cameraModel);
    Optional<MediaFile> findByFilePath(String filePath);
    
    @Query(value = "SELECT * FROM media_files WHERE exif_timestamp BETWEEN ?1 AND ?2", nativeQuery = true)
    List<MediaFile> findByExifTimestampBetween(long startTimestamp, long endTimestamp);
    
    // 复合时间查询：优先使用EXIF时间，不存在时使用创建时间
    @Query(value = "SELECT * FROM media_files WHERE " +
            "(exif_timestamp IS NOT NULL AND exif_timestamp BETWEEN ?1 AND ?2) " +
            "OR (exif_timestamp IS NULL AND create_time BETWEEN ?1 AND ?2)", nativeQuery = true)
    List<MediaFile> findByEffectiveTimeBetween(long startTimestamp, long endTimestamp);

    // 支持分页的复合排序查询：优先使用EXIF时间，不存在时使用创建时间（降序）
    @Query(value = "SELECT m.* FROM media_files m " +
            "WHERE (:filterType = 'all' OR m.file_type = :filterType) " +
            "AND (:cameraModel IS NULL OR m.camera_model = :cameraModel) " +
            "AND (:date IS NULL OR " +
            "  (m.exif_timestamp IS NOT NULL AND m.exif_timestamp BETWEEN :startTimestamp AND :endTimestamp) " +
            "  OR (m.exif_timestamp IS NULL AND m.create_time BETWEEN :startTimestamp AND :endTimestamp)) " +
            "ORDER BY CASE WHEN m.exif_timestamp IS NOT NULL THEN m.exif_timestamp ELSE m.create_time END DESC " +
            "LIMIT :size OFFSET :offset", nativeQuery = true)
    List<MediaFile> findAllWithEffectiveSortDesc(
        @Param("filterType") String filterType,
        @Param("cameraModel") String cameraModel,
        @Param("date") String date,
        @Param("startTimestamp") Long startTimestamp,
        @Param("endTimestamp") Long endTimestamp,
        @Param("size") int size,
        @Param("offset") int offset
    );
    
    // 支持分页的复合排序查询：优先使用EXIF时间，不存在时使用创建时间（升序）
    @Query(value = "SELECT m.* FROM media_files m " +
            "WHERE (:filterType = 'all' OR m.file_type = :filterType) " +
            "AND (:cameraModel IS NULL OR m.camera_model = :cameraModel) " +
            "AND (:date IS NULL OR " +
            "  (m.exif_timestamp IS NOT NULL AND m.exif_timestamp BETWEEN :startTimestamp AND :endTimestamp) " +
            "  OR (m.exif_timestamp IS NULL AND m.create_time BETWEEN :startTimestamp AND :endTimestamp)) " +
            "ORDER BY CASE WHEN m.exif_timestamp IS NOT NULL THEN m.exif_timestamp ELSE m.create_time END ASC " +
            "LIMIT :size OFFSET :offset", nativeQuery = true)
    List<MediaFile> findAllWithEffectiveSortAsc(
        @Param("filterType") String filterType,
        @Param("cameraModel") String cameraModel,
        @Param("date") String date,
        @Param("startTimestamp") Long startTimestamp,
        @Param("endTimestamp") Long endTimestamp,
        @Param("size") int size,
        @Param("offset") int offset
    );
    
    // 获取总数（用于分页）
    @Query(value = "SELECT COUNT(*) FROM media_files m " +
            "WHERE (:filterType = 'all' OR m.file_type = :filterType) " +
            "AND (:cameraModel IS NULL OR m.camera_model = :cameraModel) " +
            "AND (:date IS NULL OR " +
            "  (m.exif_timestamp IS NOT NULL AND m.exif_timestamp BETWEEN :startTimestamp AND :endTimestamp) " +
            "  OR (m.exif_timestamp IS NULL AND m.create_time BETWEEN :startTimestamp AND :endTimestamp))", nativeQuery = true)
    long countWithFilters(
        @Param("filterType") String filterType,
        @Param("cameraModel") String cameraModel,
        @Param("date") String date,
        @Param("startTimestamp") Long startTimestamp,
        @Param("endTimestamp") Long endTimestamp
    );
    
    Page<MediaFile> findAll(Pageable pageable);
    Page<MediaFile> findByCameraModel(String cameraModel, Pageable pageable);
    Page<MediaFile> findByFileType(String fileType, Pageable pageable);
    Page<MediaFile> findByCameraModelAndFileType(String cameraModel, String fileType, Pageable pageable);

    // 查询包含照片的日期列表（按拍摄时间）
    @Query(value = "SELECT strftime('%Y-%m-%d', datetime(exif_timestamp / 1000, 'unixepoch')) as date, COUNT(*) as count FROM media_files " +
            "WHERE (:filterType = 'all' OR file_type = :filterType) " +
            "AND (:cameraModel IS NULL OR camera_model = :cameraModel) " +
            "AND exif_timestamp IS NOT NULL " +
            "GROUP BY strftime('%Y-%m-%d', datetime(exif_timestamp / 1000, 'unixepoch')) " +
            "ORDER BY date DESC", nativeQuery = true)
    List<Object[]> findDatesByExifTimestamp(
        @Param("filterType") String filterType,
        @Param("cameraModel") String cameraModel
    );

    // 查询包含照片的日期列表（按创建时间）
    @Query(value = "SELECT strftime('%Y-%m-%d', datetime(create_time / 1000, 'unixepoch')) as date, COUNT(*) as count FROM media_files " +
            "WHERE (:filterType = 'all' OR file_type = :filterType) " +
            "AND (:cameraModel IS NULL OR camera_model = :cameraModel) " +
            "AND create_time IS NOT NULL " +
            "GROUP BY strftime('%Y-%m-%d', datetime(create_time / 1000, 'unixepoch')) " +
            "ORDER BY date DESC", nativeQuery = true)
    List<Object[]> findDatesByCreateTime(
        @Param("filterType") String filterType,
        @Param("cameraModel") String cameraModel
    );

    // 查询包含照片的日期列表（按修改时间）
    @Query(value = "SELECT strftime('%Y-%m-%d', datetime(modify_time / 1000, 'unixepoch')) as date, COUNT(*) as count FROM media_files " +
            "WHERE (:filterType = 'all' OR file_type = :filterType) " +
            "AND (:cameraModel IS NULL OR camera_model = :cameraModel) " +
            "AND modify_time IS NOT NULL " +
            "GROUP BY strftime('%Y-%m-%d', datetime(modify_time / 1000, 'unixepoch')) " +
            "ORDER BY date DESC", nativeQuery = true)
    List<Object[]> findDatesByModifyTime(
        @Param("filterType") String filterType,
        @Param("cameraModel") String cameraModel
    );

    // 查询包含照片的日期列表（匹配三种时间中任一时间值）
    @Query(value = "SELECT date, COUNT(*) as count FROM (" +
            "SELECT strftime('%Y-%m-%d', datetime(exif_timestamp / 1000, 'unixepoch')) as date FROM media_files " +
            "WHERE (:filterType = 'all' OR file_type = :filterType) " +
            "AND (:cameraModel IS NULL OR camera_model = :cameraModel) " +
            "AND exif_timestamp IS NOT NULL " +
            "UNION " +
            "SELECT strftime('%Y-%m-%d', datetime(create_time / 1000, 'unixepoch')) as date FROM media_files " +
            "WHERE (:filterType = 'all' OR file_type = :filterType) " +
            "AND (:cameraModel IS NULL OR camera_model = :cameraModel) " +
            "AND create_time IS NOT NULL " +
            "UNION " +
            "SELECT strftime('%Y-%m-%d', datetime(modify_time / 1000, 'unixepoch')) as date FROM media_files " +
            "WHERE (:filterType = 'all' OR file_type = :filterType) " +
            "AND (:cameraModel IS NULL OR camera_model = :cameraModel) " +
            "AND modify_time IS NOT NULL" +
            ") combined_dates " +
            "GROUP BY date " +
            "ORDER BY date DESC", nativeQuery = true)
    List<Object[]> findDatesByAnyTime(
        @Param("filterType") String filterType,
        @Param("cameraModel") String cameraModel
    );
}