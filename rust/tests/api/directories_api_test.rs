//! Directories API integration tests

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;
    use latte_album::helpers::start_test_server;
    use latte_album::config::Config;
    use latte_album::app::App;
    use tempfile::TempDir;

    /// Create a test configuration with file-based database for isolation
    async fn test_config() -> (Config, TempDir) {
        let temp_dir = tempfile::Builder::new()
            .prefix("latte_test_dirs_")
            .tempdir()
            .expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let config = Config {
            db_path,
            ..Config::default()
        };

        (config, temp_dir)
    }

    #[tokio::test]
    async fn test_list_directories_empty() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("http://{}/api/directories", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Vec<serde_json::Value> = response.json().await.unwrap();
        assert!(body.is_empty());
    }
}
