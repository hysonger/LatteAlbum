//! System API integration tests

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use reqwest::StatusCode;
    use latte_album::helpers::start_test_server;
    use latte_album::config::Config;
    use latte_album::app::App;
    use tempfile::TempDir;

    /// Create a test configuration with file-based database for isolation
    async fn test_config() -> (Config, TempDir) {
        let temp_dir = tempfile::Builder::new()
            .prefix("latte_test_sys_")
            .tempdir()
            .expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let config = Config {
            db_path,
            ..Config::default()
        };

        (config, temp_dir)
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct SystemStatus {
        status: String,
        total_files: i64,
        image_count: i64,
        video_count: i64,
        cache_size_mb: f64,
        last_scan_time: Option<String>,
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    #[serde(rename_all = "camelCase")]
    struct ScanProgress {
        status: String,
        phase: Option<String>,
        total_files: u64,
        success_count: u64,
        failure_count: u64,
        progress_percentage: String,
        files_to_add: u64,
        files_to_update: u64,
        files_to_delete: u64,
        start_time: Option<String>,
    }

    #[tokio::test]
    async fn test_get_system_status() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("http://{}/api/system/status", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: SystemStatus = response.json().await.unwrap();

        // Verify status response structure
        assert_eq!(body.status, "running");
        assert!(body.image_count >= 0);
        assert!(body.video_count >= 0);
    }

    #[tokio::test]
    async fn test_trigger_rescan() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let client = reqwest::Client::new();
        let response = client
            .post(&format!("http://{}/api/system/rescan", addr))
            .send()
            .await
            .unwrap();

        // Rescan should return success (may return 200 or 202 depending on implementation)
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_get_scan_progress_idle() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("http://{}/api/system/scan/progress", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: ScanProgress = response.json().await.unwrap();

        // Verify progress response structure - status should be "idle" when not scanning
        assert_eq!(body.status, "idle");
        assert_eq!(body.total_files, 0);
    }

    #[tokio::test]
    async fn test_cancel_scan_not_scanning() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let client = reqwest::Client::new();
        let response = client
            .post(&format!("http://{}/api/system/scan/cancel", addr))
            .send()
            .await
            .unwrap();

        // Cancel when not scanning should return success (idempotent)
        assert_eq!(response.status(), StatusCode::OK);
    }
}
