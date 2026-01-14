use tokio::sync::broadcast;
use std::sync::Arc;

/// Scan progress message
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanProgressMessage {
    pub scanning: bool,
    pub phase: Option<String>,
    pub phase_message: Option<String>,
    pub total_files: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub progress_percentage: String,
    pub status: String, // started, progress, completed, error, cancelled
    pub files_to_add: u64,
    pub files_to_update: u64,
    pub files_to_delete: u64,
}

impl Default for ScanProgressMessage {
    fn default() -> Self {
        Self {
            scanning: false,
            phase: None,
            phase_message: None,
            total_files: 0,
            success_count: 0,
            failure_count: 0,
            progress_percentage: "0.00".to_string(),
            status: "idle".to_string(),
            files_to_add: 0,
            files_to_update: 0,
            files_to_delete: 0,
        }
    }
}

/// Broadcaster for scan progress updates
pub struct ScanProgressBroadcaster {
    tx: broadcast::Sender<ScanProgressMessage>,
}

impl ScanProgressBroadcaster {
    /// Create a new broadcaster
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    /// Subscribe to progress updates
    pub fn subscribe(&self) -> broadcast::Receiver<ScanProgressMessage> {
        self.tx.subscribe()
    }

    /// Send scan started message
    pub async fn send_started(&self) {
        let msg = ScanProgressMessage {
            scanning: true,
            status: "started".to_string(),
            ..Default::default()
        };
        let _ = self.tx.send(msg);
    }

    /// Send scan completed message
    pub async fn send_completed(&self) {
        let msg = ScanProgressMessage {
            scanning: false,
            status: "completed".to_string(),
            ..Default::default()
        };
        let _ = self.tx.send(msg);
    }

    /// Send scan cancelled message
    pub async fn send_cancelled(&self) {
        let msg = ScanProgressMessage {
            scanning: false,
            status: "cancelled".to_string(),
            ..Default::default()
        };
        let _ = self.tx.send(msg);
    }

    /// Send error message
    pub async fn send_error(&self, message: &str) {
        let msg = ScanProgressMessage {
            scanning: false,
            status: "error".to_string(),
            phase_message: Some(message.to_string()),
            ..Default::default()
        };
        let _ = self.tx.send(msg);
    }

    /// Update total files
    pub async fn update_total(&self, total: u64) {
        let mut msg = self.get_current_message().await;
        msg.total_files = total;
        let _ = self.tx.send(msg);
    }

    /// Update phase information
    pub async fn update_phase(&self, phase: &str, message: &str) {
        let mut msg = self.get_current_message().await;
        msg.phase = Some(phase.to_string());
        msg.phase_message = Some(message.to_string());
        let _ = self.tx.send(msg);
    }

    /// Update progress
    pub async fn update_progress(&self, processed: u64, total: u64) {
        let mut msg = self.get_current_message().await;
        msg.success_count = processed;
        let percentage = if total > 0 {
            (processed as f64 / total as f64 * 100.0)
        } else {
            0.0
        };
        msg.progress_percentage = format!("{:.2}", percentage);
        msg.status = "progress".to_string();
        let _ = self.tx.send(msg);
    }

    /// Get current progress state
    pub async fn get_current_progress(&self) -> ScanProgressMessage {
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
