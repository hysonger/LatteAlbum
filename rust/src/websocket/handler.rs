use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use crate::websocket::broadcast::ScanProgressBroadcaster;
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Handle WebSocket connection for scan progress
pub async fn handle_websocket(ws: WebSocket, broadcaster: Arc<ScanProgressBroadcaster>) {
    let (mut sender, mut receiver) = ws.split();

    // Create channel for progress updates
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Subscribe to progress updates
    let mut progress_rx = broadcaster.subscribe();

    // Task 1: Forward progress updates to channel
    let forward_task = tokio::spawn(async move {
        while let Ok(progress) = progress_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&progress) {
                if tx.send(json).await.is_err() {
                    break;
                }
            }
        }
    });

    // Task 2: Receive from channel and websocket, forward to client
    let receive_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(json) = rx.recv() => {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Some(result) = receiver.next() => {
                    match result {
                        Ok(Message::Text(text)) => {
                            if text == "ping" {
                                let _ = sender.send(Message::Pong(vec![].into())).await;
                            }
                        }
                        Ok(Message::Ping(data)) => {
                            let _ = sender.send(Message::Pong(data)).await;
                        }
                        Ok(Message::Close(_)) => {
                            break;
                        }
                        Err(_) => {
                            break;
                        }
                        _ => {}
                    }
                }
                else => break,
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = forward_task => {},
        _ = receive_task => {},
    }
}

/// WebSocket upgrade handler
pub async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|_socket| async {})
}
