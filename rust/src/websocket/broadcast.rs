use tokio::sync::broadcast;
use std::sync::Arc;
use crate::websocket::ScanStateManager;

/// Scan progress message
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressMessage {
    pub scanning: bool,
    pub phase: Option<String>,
    // phase_message 字段已移除，由前端根据 phase 值显示中文文本
    pub total_files: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub progress_percentage: String,
    pub status: String, // started, progress, completed, error, cancelled
    pub files_to_add: u64,
    pub files_to_update: u64,
    pub files_to_delete: u64,
    pub start_time: Option<String>, // ISO timestamp for scan start
}

impl Default for ScanProgressMessage {
    fn default() -> Self {
        Self {
            scanning: false,
            phase: None,
            total_files: 0,
            success_count: 0,
            failure_count: 0,
            progress_percentage: "0.00".to_string(),
            status: "idle".to_string(),
            files_to_add: 0,
            files_to_update: 0,
            files_to_delete: 0,
            start_time: None,
        }
    }
}

/// Broadcaster for scan progress updates
#[derive(Clone)]
pub struct ScanProgressBroadcaster {
    tx: broadcast::Sender<ScanProgressMessage>,
    scan_state: Option<Arc<ScanStateManager>>,
}

impl ScanProgressBroadcaster {
    /// Create a new broadcaster
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx, scan_state: None }
    }

    /// Set the scan_state reference (must be called after creating ScanStateManager)
    pub fn set_scan_state(&mut self, scan_state: Arc<ScanStateManager>) {
        self.scan_state = Some(scan_state);
    }

    /// Subscribe to progress updates
    pub fn subscribe(&self) -> broadcast::Receiver<ScanProgressMessage> {
        self.tx.subscribe()
    }

    /// Get a sender clone for creating progress trackers
    pub fn sender(&self) -> broadcast::Sender<ScanProgressMessage> {
        self.tx.clone()
    }

    /// Get current progress state (uses shared state, not broadcast channel)
    pub async fn get_current_progress(&self) -> ScanProgressMessage {
        // Use scan_state shared state if available
        if let Some(ref state) = self.scan_state {
            return state.to_progress_message();
        }
        // Fallback to broadcast channel if scan_state not set
        self.get_current_message().await
    }


    async fn get_current_message(&self) -> ScanProgressMessage {
        // Get the latest message from the channel
        let mut rx = self.tx.subscribe();
        if let Ok(msg) = rx.try_recv() {
            msg
        } else {
            ScanProgressMessage {
                scanning: false,
                status: "idle".to_string(),
                ..Default::default()
            }
        }
    }
}

impl Default for ScanProgressBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_progress_message_default() {
        let msg = ScanProgressMessage::default();
        assert!(!msg.scanning);
        assert!(msg.phase.is_none());
        assert_eq!(msg.total_files, 0);
        assert_eq!(msg.success_count, 0);
        assert_eq!(msg.failure_count, 0);
        assert_eq!(msg.progress_percentage, "0.00");
        assert_eq!(msg.status, "idle");
    }

    #[tokio::test]
    async fn test_scan_progress_message_serde() {
        let msg = ScanProgressMessage {
            scanning: true,
            phase: Some("processing".to_string()),
            total_files: 100,
            success_count: 50,
            failure_count: 2,
            progress_percentage: "52.00".to_string(),
            status: "progress".to_string(),
            files_to_add: 30,
            files_to_update: 20,
            files_to_delete: 5,
            start_time: Some("2024-06-15T10:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"scanning\":true"));
        assert!(json.contains("\"phase\":\"processing\""));
        assert!(json.contains("\"status\":\"progress\""));
    }

    #[tokio::test]
    async fn test_scan_progress_broadcaster_new() {
        let broadcaster = ScanProgressBroadcaster::new();
        assert!(broadcaster.subscribe().try_recv().is_err());
    }

    #[tokio::test]
    async fn test_scan_progress_broadcaster_subscribe() {
        let broadcaster = ScanProgressBroadcaster::new();
        let _rx = broadcaster.subscribe();
    }

    #[tokio::test]
    async fn test_scan_progress_broadcaster_get_current_progress() {
        let broadcaster = ScanProgressBroadcaster::new();
        let progress = broadcaster.get_current_progress().await;
        assert!(!progress.scanning);
        assert_eq!(progress.status, "idle");
    }

    #[tokio::test]
    async fn test_scan_progress_broadcaster_with_scan_state() {
        let (tx, _) = broadcast::channel(100);
        let scan_state = Arc::new(ScanStateManager::new_with_interval(tx.clone(), 10));

        let mut broadcaster = ScanProgressBroadcaster::new();
        broadcaster.set_scan_state(scan_state);

        let progress = broadcaster.get_current_progress().await;
        assert!(!progress.scanning);
    }
}
