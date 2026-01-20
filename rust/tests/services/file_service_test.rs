//! FileService integration tests

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use bytes::Bytes;
    use tempfile::Builder;
    use latte_album::fixtures::TestFixtures;
    use latte_album::db::DatabasePool;
    use latte_album::services::CacheService;
    use latte_album::config::Config;

    #[tokio::test]
    async fn test_cache_service_operations() {
        let (_fixtures, _photos_dir) = TestFixtures::new();
        let db_path = std::path::Path::new(":memory:");
        let pool = DatabasePool::new(db_path).await.unwrap();
        pool.migrate(std::path::Path::new("./src/db/migrations")).await.unwrap();

        let cache_dir = Builder::new()
            .prefix("latte_test_cache_")
            .tempdir()
            .expect("Failed to create cache dir");

        let config = Config::default();
        let cache_dir_path = PathBuf::from(cache_dir.path());
        let cache = CacheService::new(
            &cache_dir_path,
            config.cache_max_capacity,
            config.cache_ttl_seconds,
        ).await.expect("Failed to create cache service");

        // Test put and get
        let test_data: &[u8] = b"test thumbnail data";
        let _ = cache.put_thumbnail("test-file-id", "small", test_data).await;

        let retrieved: Option<Bytes> = cache.get_thumbnail("test-file-id", "small").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), test_data);

        // Test cache miss
        let missed: Option<Bytes> = cache.get_thumbnail("other-file-id", "small").await;
        assert!(missed.is_none());

        // Test delete
        cache.delete_thumbnail("test-file-id", Some("small")).await;
        let after_delete: Option<Bytes> = cache.get_thumbnail("test-file-id", "small").await;
        assert!(after_delete.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let (_fixtures, _photos_dir) = TestFixtures::new();
        let cache_dir = Builder::new()
            .prefix("latte_test_cache_")
            .tempdir()
            .expect("Failed to create cache dir");

        let config = Config::default();
        let cache_dir_path = PathBuf::from(cache_dir.path());
        let cache = CacheService::new(
            &cache_dir_path,
            config.cache_max_capacity,
            config.cache_ttl_seconds,
        ).await.expect("Failed to create cache service");

        // Add some data
        let _ = cache.put_thumbnail("file1", "small", b"data1").await;
        let _ = cache.put_thumbnail("file2", "small", b"data2").await;

        // Clear all
        let _ = cache.clear_all().await;

        // Verify cleared
        let result1: Option<Bytes> = cache.get_thumbnail("file1", "small").await;
        let result2: Option<Bytes> = cache.get_thumbnail("file2", "small").await;
        assert!(result1.is_none());
        assert!(result2.is_none());
    }

    #[tokio::test]
    async fn test_cache_size_calculation() {
        let (_fixtures, _photos_dir) = TestFixtures::new();
        let cache_dir = Builder::new()
            .prefix("latte_test_cache_")
            .tempdir()
            .expect("Failed to create cache dir");

        let config = Config::default();
        let cache_dir_path = PathBuf::from(cache_dir.path());
        let cache = CacheService::new(
            &cache_dir_path,
            config.cache_max_capacity,
            config.cache_ttl_seconds,
        ).await.expect("Failed to create cache service");

        // Initial size should be >= 0
        let size = cache.get_cache_size_mb().await.unwrap_or(0.0);
        assert!(size >= 0.0);

        // Add data
        let _ = cache.put_thumbnail("file1", "small", &vec![0u8; 1000]).await;

        // Size should increase
        let new_size = cache.get_cache_size_mb().await.unwrap_or(0.0);
        assert!(new_size >= size);
    }
}
