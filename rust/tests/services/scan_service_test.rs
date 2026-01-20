//! ScanService integration tests

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use tokio::time::Duration;
    use latte_album::fixtures::TestFixtures;
    use latte_album::db::{DatabasePool, MediaFileRepository};
    use latte_album::processors::ProcessorRegistry;
    use latte_album::services::ScanService;
    use latte_album::config::Config;
    use latte_album::websocket::ScanStateManager;
    use tempfile::TempDir;

    /// Create a test configuration with file-based database for isolation
    async fn create_test_config(photos_dir: &PathBuf) -> (Config, TempDir) {
        let temp_dir = tempfile::Builder::new()
            .prefix("latte_test_scan_")
            .tempdir()
            .expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let mut config = Config::default();
        config.base_path = photos_dir.to_string_lossy().to_string().into();
        config.db_path = db_path;

        (config, temp_dir)
    }

    /// Create a test scan service
    async fn create_test_scan_service(photos_dir: &PathBuf) -> (ScanService, DatabasePool, std::sync::Arc<ScanStateManager>, TempDir) {
        let (config, temp_dir) = create_test_config(photos_dir).await;

        let db = DatabasePool::new(&config.db_path)
            .await
            .expect("Failed to create database pool");
        db.migrate(std::path::Path::new("./src/db/migrations"))
            .await
            .expect("Failed to run migrations");

        let (tx, _rx) = tokio::sync::broadcast::channel(100);
        let scan_state = std::sync::Arc::new(ScanStateManager::new(tx));
        let processors = std::sync::Arc::new(ProcessorRegistry::new(None));

        let scan_service = ScanService::new(
            config,
            db.clone(),
            processors,
            scan_state.clone(),
        );

        (scan_service, db, scan_state, temp_dir)
    }

    #[tokio::test]
    async fn test_scan_empty_directory() {
        let (_fixtures, photos_dir) = TestFixtures::new();

        let (scan_service, _, scan_state, _) = create_test_scan_service(&photos_dir).await;

        // Run scan
        scan_service.scan().await;

        // Wait for scan to complete
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Verify scan completed
        let state = scan_state.get_state();
        assert!(!state.scanning);
    }

    #[tokio::test]
    async fn test_scan_no_files() {
        let (_fixtures, photos_dir) = TestFixtures::new();

        let (scan_service, db, _, _) = create_test_scan_service(&photos_dir).await;

        // No files in directory
        scan_service.scan().await;

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Verify completed with 0 files
        let repo = MediaFileRepository::new(&db);
        let files = repo.find_all(None, None, None, None, "exif_timestamp", "desc", 0, 100)
            .await
            .unwrap();
        assert_eq!(files.len(), 0);
    }

    #[tokio::test]
    async fn test_scan_idempotent() {
        let (_fixtures, photos_dir) = TestFixtures::new();

        let (scan_service, db, _, _) = create_test_scan_service(&photos_dir).await;

        // First scan
        scan_service.scan().await;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Get initial file count
        let repo = MediaFileRepository::new(&db);
        let initial_count = repo.find_all(None, None, None, None, "exif_timestamp", "desc", 0, 1000)
            .await
            .unwrap()
            .len();

        // Second scan (should skip unchanged files)
        scan_service.scan().await;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Get file count after second scan
        let final_count = repo.find_all(None, None, None, None, "exif_timestamp", "desc", 0, 1000)
            .await
            .unwrap()
            .len();

        // Count should be the same
        assert_eq!(initial_count, final_count);
    }
}
