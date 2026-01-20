//! Files API integration tests

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
            .prefix("latte_test_files_")
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
    struct FilesResponse {
        items: Vec<serde_json::Value>,
        total: i64,
        page: i32,
        size: i32,
    }

    #[tokio::test]
    async fn test_list_files_empty() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("http://{}/api/files", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: FilesResponse = response.json().await.unwrap();
        assert_eq!(body.items.len(), 0);
        assert_eq!(body.total, 0);
    }

    #[tokio::test]
    async fn test_list_files_with_pagination() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("http://{}/api/files?page=0&size=10", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: FilesResponse = response.json().await.unwrap();
        assert_eq!(body.page, 0);
        assert_eq!(body.size, 10);
    }

    #[tokio::test]
    async fn test_get_file_details_not_found() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("http://{}/api/files/non-existent-id", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_dates_empty() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("http://{}/api/files/dates", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Vec<serde_json::Value> = response.json().await.unwrap();
        assert_eq!(body.len(), 0);
    }

    #[tokio::test]
    async fn test_list_files_filter_by_type() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("http://{}/api/files?filterType=image", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
