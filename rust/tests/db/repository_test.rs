//! Repository integration tests

#[cfg(test)]
mod tests {
    use latte_album::fixtures::{create_test_media_file, create_test_media_file_with};
    use latte_album::db::{DatabasePool, MediaFileRepository};
    use chrono::{Utc, TimeZone};

    /// Wrapper that holds the database pool and keeps the temp dir alive
    struct TestDb {
        pool: DatabasePool,
        _temp_dir: tempfile::TempDir,
    }

    /// Create a test database pool with a unique file-based database
    async fn test_db_pool() -> TestDb {
        // Use a unique temporary directory for each test to ensure isolation
        let temp_dir = tempfile::Builder::new()
            .prefix("latte_test_db_")
            .tempdir()
            .expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let pool = DatabasePool::new(&db_path)
            .await
            .expect("Failed to create database pool");
        pool.migrate(std::path::Path::new("./src/db/migrations"))
            .await
            .expect("Failed to run migrations");
        TestDb { pool, _temp_dir: temp_dir }
    }

    /// Get the database pool from TestDb
    fn get_pool(db: &TestDb) -> &DatabasePool {
        &db.pool
    }

    #[tokio::test]
    async fn test_batch_upsert() {
        let db = test_db_pool().await;
        let pool = get_pool(&db);
        let repo = MediaFileRepository::new(pool);

        let files = vec![
            create_test_media_file("test1.jpg"),
            create_test_media_file("test2.jpg"),
            create_test_media_file("test3.jpg"),
        ];

        repo.batch_upsert(&files).await.unwrap();

        let result = repo
            .find_all(None, None, None, None, "exif_timestamp", "desc", 0, 50)
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_find_all_pagination() {
        let db = test_db_pool().await;
        let pool = get_pool(&db);
        let repo = MediaFileRepository::new(pool);

        // Insert 10 files
        let files: Vec<_> = (0..10)
            .map(|i| create_test_media_file(&format!("test{}.jpg", i)))
            .collect();
        repo.batch_upsert(&files).await.unwrap();

        // Get first page
        let result = repo
            .find_all(None, None, None, None, "exif_timestamp", "desc", 0, 5)
            .await
            .unwrap();
        assert_eq!(result.len(), 5);

        // Get second page
        let result = repo
            .find_all(None, None, None, None, "exif_timestamp", "desc", 1, 5)
            .await
            .unwrap();
        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_find_all_filters() {
        let db = test_db_pool().await;
        let pool = get_pool(&db);
        let repo = MediaFileRepository::new(pool);

        // Insert test files with different types
        let files = vec![
            create_test_media_file_with("test1.jpg", "image", None),
            create_test_media_file_with("test2.png", "image", None),
            create_test_media_file_with("test3.mp4", "video", None),
        ];
        repo.batch_upsert(&files).await.unwrap();

        // Filter by image type
        let result = repo
            .find_all(None, Some("image"), None, None, "exif_timestamp", "desc", 0, 50)
            .await
            .unwrap();
        assert_eq!(result.len(), 2);

        // Filter by video type
        let result = repo
            .find_all(None, Some("video"), None, None, "exif_timestamp", "desc", 0, 50)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let db = test_db_pool().await;
        let pool = get_pool(&db);
        let repo = MediaFileRepository::new(pool);

        let file = create_test_media_file("test.jpg");
        let file_id = file.id.clone();

        repo.batch_upsert(&[file]).await.unwrap();

        let result = repo.find_by_id(&file_id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().file_name, "test.jpg");
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let db = test_db_pool().await;
        let pool = get_pool(&db);
        let repo = MediaFileRepository::new(pool);

        let result = repo.find_by_id("non-existent-id").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_dates_with_files() {
        let db = test_db_pool().await;
        let pool = get_pool(&db);
        let repo = MediaFileRepository::new(pool);

        // Insert files with different dates
        let ts1 = Utc.timestamp_opt(1700000000, 0).unwrap();
        let ts2 = Utc.timestamp_opt(1700088000, 0).unwrap();
        let ts3 = Utc.timestamp_opt(1700174400, 0).unwrap();

        let files = vec![
            create_test_media_file_with("photo1.jpg", "image", Some(ts1.naive_utc())),
            create_test_media_file_with("photo2.jpg", "image", Some(ts2.naive_utc())),
            create_test_media_file_with("photo3.jpg", "image", Some(ts3.naive_utc())),
        ];
        repo.batch_upsert(&files).await.unwrap();

        let dates = repo.find_dates_with_files(None, None).await.unwrap();

        // Should return 3 dates (all files on different days)
        assert_eq!(dates.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_missing() {
        let db = test_db_pool().await;
        let pool = get_pool(&db);
        let repo = MediaFileRepository::new(pool);

        let files = vec![
            create_test_media_file("test1.jpg"),
            create_test_media_file("test2.jpg"),
            create_test_media_file("test3.jpg"),
        ];
        repo.batch_upsert(&files).await.unwrap();

        // Delete one file from the "filesystem"
        let existing_paths = vec!["/test/photos/test1.jpg".to_string(), "/test/photos/test2.jpg".to_string()];

        repo.delete_missing(&existing_paths).await.unwrap();

        // Verify test3.jpg was deleted
        let result = repo.find_by_id(&files[2].id).await.unwrap();
        assert!(result.is_none());

        // Verify test1.jpg and test2.jpg still exist
        let result = repo.find_by_id(&files[0].id).await.unwrap();
        assert!(result.is_some());
    }
}
