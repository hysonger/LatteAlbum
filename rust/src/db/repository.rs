use crate::db::models::{DateInfo, Directory, MediaFile};
use crate::db::pool::DatabasePool;
use chrono::{NaiveDateTime, Utc};
use sqlx::Row;
use std::path::{Path, PathBuf};

/// Repository for media file database operations
pub struct MediaFileRepository<'a> {
    db: &'a DatabasePool,
}

impl<'a> MediaFileRepository<'a> {
    pub fn new(db: &'a DatabasePool) -> Self {
        Self { db }
    }

    /// Get all media files with pagination and filtering
    pub async fn find_all(
        &self,
        path_filter: Option<&str>,
        file_type: Option<&str>,
        camera_model: Option<&str>,
        date_filter: Option<&str>,
        sort_by: &str,
        order: &str,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<MediaFile>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM media_files WHERE 1=1");
        let mut params: Vec<String> = Vec::new();

        if let Some(path) = path_filter {
            query.push_str(" AND file_path LIKE ?");
            params.push(format!("%{}%", path));
        }

        if let Some(ft) = file_type {
            if ft != "all" {
                query.push_str(" AND file_type = ?");
                params.push(ft.to_string());
            }
        }

        if let Some(camera) = camera_model {
            query.push_str(" AND camera_model = ?");
            params.push(camera.to_string());
        }

        if let Some(date) = date_filter {
            query.push_str(" AND (exif_timestamp LIKE ? OR create_time LIKE ? OR modify_time LIKE ?)");
            let date_prefix = format!("{}%", date);
            params.push(date_prefix.clone());
            params.push(date_prefix.clone());
            params.push(date_prefix);
        }

        // Sort by effective time (EXIF > create > modify)
        let sort_field = match sort_by {
            "exifTimestamp" => "exif_timestamp",
            "createTime" => "create_time",
            "modifyTime" => "modify_time",
            "fileName" => "file_name",
            _ => "exif_timestamp",
        };

        query.push_str(&format!(" ORDER BY CASE WHEN {} IS NOT NULL THEN 0 ELSE 1 END, {} {}",
            sort_field, sort_field, if order == "asc" { "ASC" } else { "DESC" }));

        query.push_str(&format!(" LIMIT {} OFFSET {}", page_size, page * page_size));

        let mut sqlx_query = sqlx::query_as::<_, MediaFile>(&query);
        for param in &params {
            sqlx_query = sqlx_query.bind(param.as_str());
        }

        sqlx_query.fetch_all(self.db.get_pool()).await
    }

    /// Get file by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<MediaFile>, sqlx::Error> {
        sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files WHERE id = ?")
            .bind(id)
            .fetch_optional(self.db.get_pool())
            .await
    }

    /// Get file by path
    pub async fn find_by_path(&self, path: &Path) -> Result<Option<MediaFile>, sqlx::Error> {
        sqlx::query_as::<_, MediaFile>("SELECT * FROM media_files WHERE file_path = ?")
            .bind(path.to_string_lossy().to_string())
            .fetch_optional(self.db.get_pool())
            .await
    }

    /// Get neighbor files for navigation
    pub async fn find_neighbors(
        &self,
        _id: &str,
        sort_time: NaiveDateTime,
        before: bool,
    ) -> Result<Option<MediaFile>, sqlx::Error> {
        let op = if before { "<" } else { ">" };
        let order = if before { "DESC" } else { "ASC" };

        let query = format!(
            "SELECT * FROM media_files
             WHERE (exif_timestamp {} ? OR (exif_timestamp IS NULL AND create_time {} ?))
             ORDER BY CASE WHEN exif_timestamp IS NOT NULL THEN 0 ELSE 1 END, exif_timestamp {} NULLS LAST, create_time {} {}
             LIMIT 1",
            op, op, order, order, order
        );

        sqlx::query_as::<_, MediaFile>(&query)
            .bind(sort_time)
            .bind(sort_time)
            .fetch_optional(self.db.get_pool())
            .await
    }

    /// Get dates with photos (for calendar)
    pub async fn find_dates_with_files(
        &self,
        _path_filter: Option<&str>,
        _file_type: Option<&str>,
    ) -> Result<Vec<DateInfo>, sqlx::Error> {
        let query = String::from(
            "SELECT date AS date, COUNT(*) AS count FROM (
                SELECT DISTINCT date(exif_timestamp) AS date FROM media_files WHERE exif_timestamp IS NOT NULL
                UNION
                SELECT DISTINCT date(create_time) AS date FROM media_files WHERE create_time IS NOT NULL AND exif_timestamp IS NULL
                UNION
                SELECT DISTINCT date(modify_time) AS date FROM media_files WHERE modify_time IS NOT NULL AND exif_timestamp IS NULL AND create_time IS NULL
            ) GROUP BY date ORDER BY date DESC"
        );

        let sqlx_query = sqlx::query_as::<_, DateInfo>(&query);

        sqlx_query.fetch_all(self.db.get_pool()).await
    }

    /// Insert or update a media file
    pub async fn upsert(&self, file: &MediaFile) -> Result<(), sqlx::Error> {
        let now = Utc::now().naive_utc();

        sqlx::query(
            "INSERT OR REPLACE INTO media_files (
                id, file_path, file_name, file_type, mime_type, file_size,
                width, height, exif_timestamp, exif_timezone_offset,
                create_time, modify_time, last_scanned,
                camera_make, camera_model, lens_model,
                exposure_time, aperture, iso, focal_length,
                duration, video_codec, thumbnail_generated
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&file.id)
        .bind(&file.file_path)
        .bind(&file.file_name)
        .bind(&file.file_type)
        .bind(&file.mime_type)
        .bind(file.file_size)
        .bind(file.width)
        .bind(file.height)
        .bind(file.exif_timestamp)
        .bind(&file.exif_timezone_offset)
        .bind(file.create_time)
        .bind(file.modify_time)
        .bind(now)
        .bind(&file.camera_make)
        .bind(&file.camera_model)
        .bind(&file.lens_model)
        .bind(&file.exposure_time)
        .bind(&file.aperture)
        .bind(file.iso)
        .bind(&file.focal_length)
        .bind(file.duration)
        .bind(&file.video_codec)
        .bind(if file.thumbnail_generated { 1 } else { 0 })
        .execute(self.db.get_pool())
        .await?;

        Ok(())
    }

    /// Delete a media file by ID
    pub async fn delete_by_id(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM media_files WHERE id = ?")
            .bind(id)
            .execute(self.db.get_pool())
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete files not in the given path list using batch DELETE
    /// Uses DELETE ... WHERE NOT IN (...) for efficient batch operation
    pub async fn delete_missing(&self, existing_paths: &[String]) -> Result<u64, sqlx::Error> {
        use sqlx::QueryBuilder;
        use sqlx::Sqlite;

        // 如果没有现有文件，删除所有记录
        if existing_paths.is_empty() {
            let result = sqlx::query("DELETE FROM media_files WHERE last_scanned IS NOT NULL")
                .execute(self.db.get_pool())
                .await?;
            tracing::debug!("delete_missing: deleted {} files (all)", result.rows_affected());
            return Ok(result.rows_affected());
        }

        // SQLite parameter limit: 32766
        // Each path uses 1 parameter for NOT IN clause
        const MAX_PARAMS: usize = 32766;
        const MAX_PATHS: usize = MAX_PARAMS;

        let mut total_deleted = 0u64;

        // Process in batches to stay within SQLite parameter limits
        for chunk in existing_paths.chunks(MAX_PATHS) {
            let mut query_builder: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
                "DELETE FROM media_files WHERE last_scanned IS NOT NULL AND file_path NOT IN "
            );

            query_builder.push_tuples(chunk.iter(), |mut b, path| {
                b.push_bind(path.as_str());
            });

            let query = query_builder.build();
            let result = query.execute(self.db.get_pool()).await?;
            total_deleted += result.rows_affected();
        }

        tracing::debug!("delete_missing: {} files deleted", total_deleted);
        Ok(total_deleted)
    }

    /// Count files with filters
    pub async fn count(
        &self,
        path_filter: Option<&str>,
        file_type: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let mut query = String::from("SELECT COUNT(*) FROM media_files WHERE 1=1");
        let mut params: Vec<String> = Vec::new();

        if let Some(path) = path_filter {
            query.push_str(" AND file_path LIKE ?");
            params.push(format!("%{}%", path));
        }

        if let Some(ft) = file_type {
            if ft != "all" {
                query.push_str(" AND file_type = ?");
                params.push(ft.to_string());
            }
        }

        let mut sqlx_query = sqlx::query_scalar::<_, i64>(&query);
        for param in &params {
            sqlx_query = sqlx_query.bind(param.as_str());
        }

        sqlx_query.fetch_one(self.db.get_pool()).await
    }

    /// Update thumbnail generated status
    pub async fn update_thumbnail_status(&self, id: &str, generated: bool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE media_files SET thumbnail_generated = ? WHERE id = ?")
            .bind(if generated { 1 } else { 0 })
            .bind(id)
            .execute(self.db.get_pool())
            .await?;

        Ok(())
    }

    /// Check if database is empty (no files scanned yet)
    pub async fn is_empty(&self) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM media_files")
            .fetch_one(self.db.get_pool())
            .await?;
        Ok(count == 0)
    }

    /// Batch check file existence - returns HashMap of path -> Option<MediaFile>
    /// Uses individual queries to avoid SQLx dynamic query issues
    pub async fn batch_find_by_paths(&self, paths: &[PathBuf]) -> Result<std::collections::HashMap<String, Option<MediaFile>>, sqlx::Error> {
        use std::collections::HashMap;

        if paths.is_empty() {
            return Ok(HashMap::new());
        }

        let mut result: HashMap<String, Option<MediaFile>> = HashMap::new();

        // Query each path individually to avoid SQLx dynamic query issues
        for path in paths {
            let path_str = path.to_string_lossy().to_string();

            match sqlx::query_as::<_, MediaFile>(
                "SELECT * FROM media_files WHERE file_path = ?"
            )
            .bind(path_str.as_str())
            .fetch_optional(self.db.get_pool())
            .await
            {
                Ok(Some(file)) => {
                    result.insert(path_str.clone(), Some(file));
                }
                Ok(None) => {
                    result.insert(path_str.clone(), None);
                }
                Err(e) => {
                    tracing::warn!("Query failed for path {}: {}", path_str, e);
                    result.insert(path_str.clone(), None);
                }
            }
        }

        let total_found = result.values().filter(|v| v.is_some()).count();
        let total_missing = result.values().filter(|v| v.is_none()).count();

        tracing::debug!("batch_find_by_paths: {} paths, {} found, {} not found",
            paths.len(), total_found, total_missing);

        Ok(result)
    }

    /// Batch check file existence using single SQL query with IN clause
    /// Uses QueryBuilder for efficient bulk SELECT
    pub async fn batch_find_by_paths_batch(&self, paths: &[PathBuf]) -> Result<Vec<MediaFile>, sqlx::Error> {
        use sqlx::QueryBuilder;
        use sqlx::Sqlite;

        if paths.is_empty() {
            return Ok(vec![]);
        }

        // For very large batches, we need to chunk to avoid SQLite parameter limits
        // SQLite: 32766 parameters max, each path uses 1 parameter
        const MAX_PARAMS: usize = 32766;
        const MAX_PATHS: usize = MAX_PARAMS;

        // Collect owned strings first to avoid lifetime issues with chunks
        let path_strings: Vec<String> = paths.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let mut all_files: Vec<MediaFile> = Vec::new();

        // Use standard slice chunks
        for chunk in path_strings.chunks(MAX_PATHS) {
            let mut query_builder: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
                "SELECT * FROM media_files WHERE file_path IN "
            );

            query_builder.push_tuples(chunk.iter(), |mut b, path| {
                b.push_bind(path.as_str());
            });

            let query = query_builder.build_query_as::<MediaFile>();
            let files = query.fetch_all(self.db.get_pool()).await?;
            all_files.extend(files);
        }

        tracing::debug!("batch_find_by_paths_batch: {} paths, {} files found",
            paths.len(), all_files.len());

        Ok(all_files)
    }

    /// Batch upsert files using QueryBuilder for efficient bulk INSERT
    pub async fn batch_upsert(&self, files: &[MediaFile]) -> Result<(), sqlx::Error> {
        use sqlx::QueryBuilder;
        use sqlx::Sqlite;

        if files.is_empty() {
            return Ok(());
        }

        // SQLite parameter limit: 32766
        // Each file uses 23 parameters, so max ~1424 files per batch
        const MAX_PARAMS: usize = 32766;
        const FIELDS_PER_FILE: usize = 23;
        const MAX_FILES_PER_BATCH: usize = MAX_PARAMS / FIELDS_PER_FILE;

        let mut tx = self.db.get_pool().begin().await?;
        let now = Utc::now().naive_utc();

        // Process in batches to stay within SQLite parameter limits
        for chunk in files.chunks(MAX_FILES_PER_BATCH) {
            // push_values automatically adds VALUES keyword
            let mut query_builder: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
                "INSERT OR REPLACE INTO media_files (
                    id, file_path, file_name, file_type, mime_type, file_size,
                    width, height, exif_timestamp, exif_timezone_offset,
                    create_time, modify_time, last_scanned,
                    camera_make, camera_model, lens_model,
                    exposure_time, aperture, iso, focal_length,
                    duration, video_codec, thumbnail_generated
                ) "
            );

            query_builder.push_values(chunk.iter(), |mut b, file| {
                b.push_bind(&file.id)
                    .push_bind(&file.file_path)
                    .push_bind(&file.file_name)
                    .push_bind(&file.file_type)
                    .push_bind(&file.mime_type)
                    .push_bind(file.file_size)
                    .push_bind(file.width)
                    .push_bind(file.height)
                    .push_bind(file.exif_timestamp)
                    .push_bind(file.exif_timezone_offset.clone())
                    .push_bind(file.create_time)
                    .push_bind(file.modify_time)
                    .push_bind(now)
                    .push_bind(file.camera_make.clone())
                    .push_bind(file.camera_model.clone())
                    .push_bind(file.lens_model.clone())
                    .push_bind(file.exposure_time.clone())
                    .push_bind(file.aperture.clone())
                    .push_bind(file.iso)
                    .push_bind(file.focal_length.clone())
                    .push_bind(file.duration)
                    .push_bind(file.video_codec.clone())
                    .push_bind(if file.thumbnail_generated { 1 } else { 0 });
            });

            let query = query_builder.build();
            query.execute(tx.as_mut()).await?;
        }

        tx.commit().await?;

        tracing::debug!("batch_upsert: {} files inserted/updated", files.len());
        Ok(())
    }

    /// Batch update last_scanned for files using QueryBuilder for efficient bulk UPDATE
    /// Uses UPDATE ... WHERE IN (...) for batch operation
    pub async fn batch_touch(&self, paths: &[PathBuf]) -> Result<u64, sqlx::Error> {
        use sqlx::QueryBuilder;
        use sqlx::Sqlite;

        if paths.is_empty() {
            return Ok(0);
        }

        // SQLite parameter limit: 32766
        // Each path uses 1 parameter for IN clause, plus 1 for last_scanned
        const MAX_PARAMS: usize = 32766;
        const MAX_PATHS: usize = MAX_PARAMS - 1;  // Reserve one for last_scanned

        let path_strings: Vec<String> = paths.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let mut total_updated = 0u64;
        let now = Utc::now().naive_utc();

        // Process in batches to stay within SQLite parameter limits
        for chunk in path_strings.chunks(MAX_PATHS) {
            let mut query_builder: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
                "UPDATE media_files SET last_scanned = "
            );
            query_builder.push_bind(now);
            query_builder.push(" WHERE file_path IN ");

            query_builder.push_tuples(chunk.iter(), |mut b, path| {
                b.push_bind(path.as_str());
            });

            let query = query_builder.build();
            let result = query.execute(self.db.get_pool()).await?;
            total_updated += result.rows_affected();
        }

        tracing::debug!("batch_touch: {} paths updated", total_updated);
        Ok(total_updated)
    }

    /// Count files in database that are not in the given path list
    /// Used to determine how many files will be deleted during scan
    pub async fn count_missing(&self, existing_paths: &[PathBuf]) -> Result<u64, sqlx::Error> {
        use std::collections::HashSet;

        if existing_paths.is_empty() {
            // If no paths exist, all files in DB are missing
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM media_files")
                .fetch_one(self.db.get_pool())
                .await?;
            return Ok(count as u64);
        }

        // Get all file paths from database that have been scanned
        let all_db_files: Vec<String> = sqlx::query_scalar(
            "SELECT file_path FROM media_files WHERE last_scanned IS NOT NULL"
        )
            .fetch_all(self.db.get_pool())
            .await?;

        // Convert existing_paths to owned Strings for HashSet
        let existing_set: HashSet<String> = existing_paths.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        // Count files in DB but not in filesystem
        let missing_count = all_db_files.iter()
            .filter(|p| !existing_set.contains(p.as_str()))
            .count() as u64;

        Ok(missing_count)
    }
}

/// Repository for directory operations
pub struct DirectoryRepository<'a> {
    db: &'a DatabasePool,
}

impl<'a> DirectoryRepository<'a> {
    pub fn new(db: &'a DatabasePool) -> Self {
        Self { db }
    }

    /// Get all directories
    pub async fn find_all(&self) -> Result<Vec<Directory>, sqlx::Error> {
        sqlx::query_as::<_, Directory>("SELECT * FROM directories ORDER BY path")
            .fetch_all(self.db.get_pool())
            .await
    }

    /// Get directory tree
    pub async fn find_tree(&self) -> Result<Vec<Directory>, sqlx::Error> {
        self.find_all().await
    }

    /// Upsert a directory
    pub async fn upsert(&self, dir: &Directory) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO directories (id, path, parent_id, last_modified)
             VALUES (?, ?, ?, ?)"
        )
        .bind(dir.id)
        .bind(&dir.path)
        .bind(dir.parent_id)
        .bind(dir.last_modified.map(|t| t.to_string()))
        .execute(self.db.get_pool())
        .await?;

        Ok(())
    }

    /// Delete directories not in the given path list
    pub async fn delete_missing(&self, existing_paths: &[String]) -> Result<u64, sqlx::Error> {
        use std::collections::HashSet;

        // 如果没有现有目录，删除所有目录
        if existing_paths.is_empty() {
            let result = sqlx::query("DELETE FROM directories")
                .execute(self.db.get_pool())
                .await?;
            return Ok(result.rows_affected());
        }

        // 将 existing_paths 转为 HashSet 用于快速查找
        let existing_set: HashSet<&str> = existing_paths.iter().map(|s| s.as_str()).collect();

        // 查询所有需要保留的目录路径
        let existing_dirs: Vec<String> = sqlx::query_scalar::<_, String>(
            "SELECT path FROM directories"
        )
        .fetch_all(self.db.get_pool())
        .await?;

        // 逐条删除不存在的目录
        let mut deleted = 0;
        for path in existing_dirs {
            if !existing_set.contains(path.as_str()) {
                sqlx::query("DELETE FROM directories WHERE path = ?")
                    .bind(&path)
                    .execute(self.db.get_pool())
                    .await?;
                deleted += 1;
            }
        }

        Ok(deleted)
    }
}
