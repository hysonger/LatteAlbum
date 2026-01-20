//! Test helper utilities for integration tests

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::time::Duration;
use latte_album::app::App;

/// Start a test server and return the address and shutdown signal
pub async fn start_test_server(app: &App) -> (SocketAddr, oneshot::Sender<()>) {
    let (tx, rx) = oneshot::channel::<()>();

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    let addr = listener.local_addr().expect("Failed to get local address");

    let router = app.router_clone();

    tokio::spawn(async move {
        let server = axum::serve(listener, router);

        tokio::select! {
            _ = server => {}
            _ = rx => {}
        }
    });

    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    (addr, tx)
}

/// Wait for a condition to be true with timeout
pub async fn wait_for_condition<F, Fut>(max_attempts: u32, delay: Duration, condition: F) -> bool
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    for _ in 0..max_attempts {
        if condition().await {
            return true;
        }
        tokio::time::sleep(delay).await;
    }
    false
}

/// Assert that a response is successful (2xx status code)
pub async fn assert_response_success(response: reqwest::Response) {
    if !response.status().is_success() {
        panic!(
            "Expected successful response, got {}",
            response.status()
        );
    }
}

/// Assert that a response has a specific status code
pub async fn assert_response_status(response: reqwest::Response, expected: reqwest::StatusCode) {
    assert_eq!(
        response.status(),
        expected,
        "Expected status {}, got {}",
        expected,
        response.status()
    );
}
