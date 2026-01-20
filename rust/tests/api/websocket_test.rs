//! WebSocket integration tests

#[cfg(test)]
mod tests {
    use latte_album::config::Config;
    use latte_album::app::App;
    use latte_album::helpers::start_test_server;
    use tempfile::TempDir;

    /// Create a test configuration with file-based database for isolation
    async fn test_config() -> (Config, TempDir) {
        let temp_dir = tempfile::Builder::new()
            .prefix("latte_test_ws_")
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
    async fn test_websocket_connection() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        // Connect to WebSocket
        let url = format!("ws://{}/ws/scan", addr);
        let result = tokio_tungstenite::connect_async(url).await;

        assert!(result.is_ok(), "WebSocket connection should succeed");
    }

    #[tokio::test]
    async fn test_websocket_multiple_connections() {
        let (config, _temp_dir) = test_config().await;
        let app = App::new(config).await.expect("Failed to create app");
        let (addr, _shutdown) = start_test_server(&app).await;

        let url = format!("ws://{}/ws/scan", addr);

        // Connect multiple clients
        let result1 = tokio_tungstenite::connect_async(&url).await;
        let result2 = tokio_tungstenite::connect_async(&url).await;
        let result3 = tokio_tungstenite::connect_async(&url).await;

        assert!(result1.is_ok(), "Client 1 should connect");
        assert!(result2.is_ok(), "Client 2 should connect");
        assert!(result3.is_ok(), "Client 3 should connect");
    }
}
