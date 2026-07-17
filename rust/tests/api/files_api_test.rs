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
            .get(format!("http://{}/api/files", addr))
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
            .get(format!("http://{}/api/files?page=0&size=10", addr))
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
            .get(format!("http://{}/api/files/non-existent-id", addr))
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
            .get(format!("http://{}/api/files/dates", addr))
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
            .get(format!("http://{}/api/files?filterType=image", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    /// GPS 坐标是敏感信息：列表与详情接口绝不能在 JSON 中出现 gpsLatitude/gpsLongitude。
    /// 直接写入一条带 GPS 的记录到数据库，再调用 API 验证字段被过滤。
    #[tokio::test]
    async fn test_list_files_never_exposes_gps() {
        use latte_album::db::{DatabasePool, MediaFileRepository};

        let (config, _temp_dir) = test_config().await;
        // 先让 App 创建并迁移好数据库
        let app = App::new(config.clone()).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        // 直接通过 repository 写入一条带 GPS 的记录
        let db = DatabasePool::new(&config.db_path).await.expect("open db");
        let repo = MediaFileRepository::new(&db);
        let mut file = latte_album::fixtures::create_test_media_file("gps-hidden.jpg");
        file.gps_latitude = Some(39.903333);
        file.gps_longitude = Some(116.391667);
        repo.upsert(&file).await.expect("upsert");

        let client = reqwest::Client::new();

        // 列表接口
        let list_resp = client
            .get(format!("http://{}/api/files", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(list_resp.status(), StatusCode::OK);
        let list_body: FilesResponse = list_resp.json().await.unwrap();
        assert_eq!(list_body.items.len(), 1);
        let item = &list_body.items[0];
        assert!(
            item.get("gpsLatitude").is_none() && item.get("gpsLongitude").is_none(),
            "GPS 字段在列表响应中泄露: {}", item
        );

        // 详情接口
        let detail_resp = client
            .get(format!("http://{}/api/files/{}", addr, file.id))
            .send()
            .await
            .unwrap();
        assert_eq!(detail_resp.status(), StatusCode::OK);
        let detail_body: serde_json::Value = detail_resp.json().await.unwrap();
        assert!(
            detail_body.get("gpsLatitude").is_none() && detail_body.get("gpsLongitude").is_none(),
            "GPS 字段在详情响应中泄露: {}", detail_body
        );
    }

    /// 对不存在的 ID 请求 GPS 应返回 404。
    #[tokio::test]
    async fn test_get_gps_not_found() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{}/api/files/nonexistent-id/gps", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// 对存在但没有 GPS 的照片，/gps 端点应返回 hasGps=false。
    #[tokio::test]
    async fn test_get_gps_no_data() {
        use latte_album::db::{DatabasePool, MediaFileRepository};

        let (config, _temp_dir) = test_config().await;
        let app = App::new(config.clone()).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let db = DatabasePool::new(&config.db_path).await.expect("open db");
        let repo = MediaFileRepository::new(&db);
        let file = latte_album::fixtures::create_test_media_file("no-gps.jpg");
        let file_id = file.id.clone();
        repo.upsert(&file).await.expect("upsert");

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{}/api/files/{}/gps", addr, file_id))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body.get("hasGps").and_then(|v| v.as_bool()), Some(false));
        assert!(body.get("latitude").is_none());
        assert!(body.get("longitude").is_none());
    }

    /// 对存在且带 GPS 的照片，/gps 端点返回坐标。
    #[tokio::test]
    async fn test_get_gps_returns_coordinates() {
        use latte_album::db::{DatabasePool, MediaFileRepository};

        let (config, _temp_dir) = test_config().await;
        let app = App::new(config.clone()).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let db = DatabasePool::new(&config.db_path).await.expect("open db");
        let repo = MediaFileRepository::new(&db);
        let mut file = latte_album::fixtures::create_test_media_file("has-gps.jpg");
        file.gps_latitude = Some(39.903333);
        file.gps_longitude = Some(116.391667);
        let file_id = file.id.clone();
        repo.upsert(&file).await.expect("upsert");

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{}/api/files/{}/gps", addr, file_id))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body.get("hasGps").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(body.get("latitude").and_then(|v| v.as_f64()), Some(39.903333));
        assert_eq!(body.get("longitude").and_then(|v| v.as_f64()), Some(116.391667));
    }
}
