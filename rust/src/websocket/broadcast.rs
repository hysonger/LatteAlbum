use tokio::sync::broadcast;

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
    pub start_time: Option<String>, // ISO timestamp for scan start
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
            start_time: None,
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

    /// Get a sender clone for creating progress trackers
    pub fn sender(&self) -> broadcast::Sender<ScanProgressMessage> {
        self.tx.clone()
    }

    /// Send scan started message
    /// phase and phase_message should come from the current phase (e.g., "processing")
    pub async fn send_started(
        &self,
        files_to_add: u64,
        files_to_update: u64,
        files_to_delete: u64,
        total_files: u64,
        phase: &str,
        phase_message: &str,
    ) {
        let start_time = chrono::Utc::now().to_rfc3339();

        let msg = ScanProgressMessage {
            scanning: true,
            status: "started".to_string(),
            files_to_add,
            files_to_update,
            files_to_delete,
            total_files,
            start_time: Some(start_time),
            phase: Some(phase.to_string()),
            phase_message: Some(phase_message.to_string()),
            ..Default::default()
        };
        let _ = self.tx.send(msg);
    }

    /// Send scan completed message
    /// Preserves the last phase information
    pub async fn send_completed(&self) {
        let mut msg = self.get_current_message().await;
        msg.scanning = false;
        msg.status = "completed".to_string();
        // Keep the phase and phase_message from the current message
        let _ = self.tx.send(msg);
    }

    /// Send scan cancelled message
    pub async fn send_cancelled(&self) {
        let mut msg = self.get_current_message().await;
        msg.scanning = false;
        msg.status = "cancelled".to_string();
        let _ = self.tx.send(msg);
    }

    /// Send error message
    pub async fn send_error(&self, message: &str) {
        let mut msg = self.get_current_message().await;
        msg.scanning = false;
        msg.status = "error".to_string();
        msg.phase_message = Some(message.to_string());
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
        msg.scanning = true;
        let _ = self.tx.send(msg);
    }

    /// Update phase information with total files
    pub async fn update_phase_with_total(&self, phase: &str, message: &str, total_files: u64) {
        let mut msg = self.get_current_message().await;
        msg.phase = Some(phase.to_string());
        msg.phase_message = Some(message.to_string());
        msg.scanning = true;
        msg.total_files = total_files;
        let _ = self.tx.send(msg);
    }

    /// Update progress with success and failure counts (can be called from sync context)
    pub fn send_progress(&self, success_count: u64, failure_count: u64, total: u64) {
        let mut msg = self.get_current_progress_sync();
        msg.success_count = success_count;
        msg.failure_count = failure_count;
        let processed = success_count + failure_count;
        let percentage = if total > 0 {
            processed as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        msg.progress_percentage = format!("{:.2}", percentage);
        msg.status = "progress".to_string();
        let _ = self.tx.send(msg);
    }

    /// Get current progress state (sync version for use in non-async contexts)
    pub fn get_current_progress_sync(&self) -> ScanProgressMessage {
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
