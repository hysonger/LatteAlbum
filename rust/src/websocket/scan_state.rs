use tokio::sync::{broadcast, mpsc};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::task::AbortHandle;
use crate::websocket::broadcast::ScanProgressMessage;

/// 扫描阶段
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanPhase {
    Idle,
    Collecting,
    Counting,
    Processing,
    Writing,
    Deleting,
    Completed,
    Error,
    Cancelled,
}

impl Default for ScanPhase {
    fn default() -> Self {
        ScanPhase::Idle
    }
}

/// 扫描状态
#[derive(Debug, Clone, Default)]
pub struct ScanState {
    pub phase: ScanPhase,
    pub scanning: bool,
    pub total_files: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub files_to_add: u64,
    pub files_to_update: u64,
    pub files_to_delete: u64,
    pub start_time: Option<String>,
}

/// 进度更新消息（业务逻辑发送的消息）
#[derive(Debug)]
pub enum ProgressUpdate {
    SetPhase(ScanPhase),
    SetTotal(u64),
    IncrementSuccess,
    IncrementFailure,
    SetFileCounts(u64, u64, u64), // add, update, delete
    ResetCounters,  // 仅重置计数器，不发送广播
    Started,
    Completed,
    Error,
    Cancelled,
}

/// 扫描状态管理器
pub struct ScanStateManager {
    state: Arc<RwLock<ScanState>>,
    progress_sender: mpsc::Sender<ProgressUpdate>,
    _worker_task: AbortHandle,
    broadcast_interval: Arc<AtomicU64>,
}

impl ScanStateManager {
    /// 创建新的状态管理器（使用默认广播间隔10）
    pub fn new(tx: broadcast::Sender<ScanProgressMessage>) -> Self {
        Self::new_with_interval(tx, 10)
    }

    /// 创建新的状态管理器（带可配置的广播间隔）
    pub fn new_with_interval(tx: broadcast::Sender<ScanProgressMessage>, broadcast_interval: u64) -> Self {
        let state = Arc::new(RwLock::new(ScanState::default()));
        let (progress_tx, mut progress_rx) = mpsc::channel(1000);
        let worker_state = state.clone();
        let tx_clone = tx.clone();
        let interval_arc = Arc::new(AtomicU64::new(broadcast_interval));

        // Clone for the worker task
        let worker_interval = interval_arc.clone();

        // Worker 任务：接收更新消息，更新状态，广播进度
        let worker_task = tokio::spawn(async move {
            let mut last_progress_reported: u64 = 0;
            let interval = worker_interval.load(Ordering::Relaxed);

            while let Some(update) = progress_rx.recv().await {
                {
                    let mut current_state = worker_state.write().unwrap();

                    match update {
                        ProgressUpdate::SetPhase(ref phase) => {
                            current_state.phase = phase.clone();
                        }
                        ProgressUpdate::SetTotal(total) => {
                            current_state.total_files = total;
                        }
                        ProgressUpdate::IncrementSuccess => {
                            current_state.success_count += 1;
                        }
                        ProgressUpdate::IncrementFailure => {
                            current_state.failure_count += 1;
                        }
                        ProgressUpdate::SetFileCounts(add, update, delete) => {
                            current_state.files_to_add = add;
                            current_state.files_to_update = update;
                            current_state.files_to_delete = delete;
                        }
                        ProgressUpdate::ResetCounters => {
                            // 仅重置计数器，不发送广播消息
                            current_state.success_count = 0;
                            current_state.failure_count = 0;
                        }
                        ProgressUpdate::Started => {
                            current_state.scanning = true;
                            current_state.start_time = Some(chrono::Utc::now().to_rfc3339());
                            current_state.success_count = 0;
                            current_state.failure_count = 0;
                        }
                        ProgressUpdate::Completed => {
                            current_state.scanning = false;
                            current_state.phase = ScanPhase::Completed;
                        }
                        ProgressUpdate::Error => {
                            current_state.scanning = false;
                            current_state.phase = ScanPhase::Error;
                        }
                        ProgressUpdate::Cancelled => {
                            current_state.scanning = false;
                            current_state.phase = ScanPhase::Cancelled;
                        }
                    }

                    // 计算进度百分比
                    let processed = current_state.success_count + current_state.failure_count;
                    let percentage = if current_state.total_files > 0 {
                        format!("{:.2}", processed as f64 / current_state.total_files as f64 * 100.0)
                    } else {
                        "0.00".to_string()
                    };

                    // 每 N 个文件发送一次进度消息，或在阶段变更/完成时发送
                    // 注意：Idle 状态不发送广播消息，避免新连接收到历史消息
                    let should_send = matches!(
                        update,
                        ProgressUpdate::SetPhase(_)
                            | ProgressUpdate::Started
                            | ProgressUpdate::Completed
                            | ProgressUpdate::Error
                            | ProgressUpdate::Cancelled
                    ) || processed.saturating_sub(last_progress_reported) >= interval;

                    if should_send {
                        // 对于完成/错误/取消状态，先保存要广播的 phase
                        let broadcast_phase = if matches!(update, ProgressUpdate::Completed | ProgressUpdate::Error | ProgressUpdate::Cancelled) {
                            current_state.phase.clone()
                        } else {
                            current_state.phase.clone()
                        };

                        let phase_str = format!("{:?}", broadcast_phase);
                        let scanning = current_state.scanning;
                        let msg = ScanProgressMessage {
                            scanning,
                            phase: Some(phase_str.clone()),
                            total_files: current_state.total_files,
                            success_count: current_state.success_count,
                            failure_count: current_state.failure_count,
                            progress_percentage: percentage,
                            status: Self::status_from_phase(&broadcast_phase),
                            files_to_add: current_state.files_to_add,
                            files_to_update: current_state.files_to_update,
                            files_to_delete: current_state.files_to_delete,
                            start_time: current_state.start_time.clone(),
                        };
                        let _ = tx_clone.send(msg);
                        last_progress_reported = processed;

                        // 广播完成后，将状态重置为 Idle，避免 broadcast channel 保存完成状态
                        // 这样新连接不会收到历史完成消息
                        if matches!(update, ProgressUpdate::Completed | ProgressUpdate::Error | ProgressUpdate::Cancelled) {
                            current_state.phase = ScanPhase::Idle;
                            current_state.scanning = false;
                            current_state.total_files = 0;
                            current_state.success_count = 0;
                            current_state.failure_count = 0;
                            current_state.files_to_add = 0;
                            current_state.files_to_update = 0;
                            current_state.files_to_delete = 0;
                            current_state.start_time = None;
                        }
                    }
                }
            }
        });

        Self {
            state,
            progress_sender: progress_tx,
            _worker_task: worker_task.abort_handle(),
            broadcast_interval: interval_arc,
        }
    }

    /// 设置广播间隔
    pub fn set_broadcast_interval(&self, interval: u64) {
        self.broadcast_interval.store(interval, Ordering::Relaxed);
    }

    /// 业务逻辑调用的接口
    pub fn set_phase(&self, phase: ScanPhase) {
        let _ = self.progress_sender.try_send(ProgressUpdate::SetPhase(phase));
    }

    pub fn set_total(&self, total: u64) {
        let _ = self.progress_sender.try_send(ProgressUpdate::SetTotal(total));
    }

    pub fn increment_success(&self) {
        let _ = self.progress_sender.try_send(ProgressUpdate::IncrementSuccess);
    }

    pub fn increment_failure(&self) {
        let _ = self.progress_sender.try_send(ProgressUpdate::IncrementFailure);
    }

    pub fn set_file_counts(&self, add: u64, update: u64, delete: u64) {
        let _ = self.progress_sender.try_send(ProgressUpdate::SetFileCounts(add, update, delete));
    }

    /// 重置计数器（仅内部状态，不发送广播）
    pub fn reset_counters(&self) {
        let _ = self.progress_sender.try_send(ProgressUpdate::ResetCounters);
    }

    pub fn started(&self) {
        let _ = self.progress_sender.try_send(ProgressUpdate::Started);
    }

    pub fn completed(&self) {
        let _ = self.progress_sender.try_send(ProgressUpdate::Completed);
    }

    pub fn error(&self) {
        let _ = self.progress_sender.try_send(ProgressUpdate::Error);
    }

    pub fn cancelled(&self) {
        let _ = self.progress_sender.try_send(ProgressUpdate::Cancelled);
    }

    /// 获取当前状态（用于查询）
    pub fn get_state(&self) -> ScanState {
        self.state.read().unwrap().clone()
    }

    /// 将当前状态转换为 ScanProgressMessage（用于 get_current_progress）
    pub fn to_progress_message(&self) -> ScanProgressMessage {
        let state = self.state.read().unwrap();
        let percentage = if state.total_files > 0 {
            format!("{:.2}", (state.success_count + state.failure_count) as f64 / state.total_files as f64 * 100.0)
        } else {
            "0.00".to_string()
        };
        ScanProgressMessage {
            scanning: state.scanning,
            phase: Some(format!("{:?}", state.phase)),
            total_files: state.total_files,
            success_count: state.success_count,
            failure_count: state.failure_count,
            progress_percentage: percentage,
            status: Self::status_from_phase(&state.phase),
            files_to_add: state.files_to_add,
            files_to_update: state.files_to_update,
            files_to_delete: state.files_to_delete,
            start_time: state.start_time.clone(),
        }
    }

    fn status_from_phase(phase: &ScanPhase) -> String {
        match phase {
            ScanPhase::Idle => "idle".to_string(),
            ScanPhase::Collecting | ScanPhase::Counting | ScanPhase::Processing | ScanPhase::Writing | ScanPhase::Deleting => {
                "progress".to_string()
            }
            ScanPhase::Completed => "completed".to_string(),
            ScanPhase::Error => "error".to_string(),
            ScanPhase::Cancelled => "cancelled".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_phase_variants() {
        assert_eq!(ScanPhase::Idle, ScanPhase::Idle);
        assert_eq!(ScanPhase::Collecting, ScanPhase::Collecting);
        assert_eq!(ScanPhase::Counting, ScanPhase::Counting);
        assert_eq!(ScanPhase::Processing, ScanPhase::Processing);
        assert_eq!(ScanPhase::Writing, ScanPhase::Writing);
        assert_eq!(ScanPhase::Deleting, ScanPhase::Deleting);
        assert_eq!(ScanPhase::Completed, ScanPhase::Completed);
        assert_eq!(ScanPhase::Error, ScanPhase::Error);
        assert_eq!(ScanPhase::Cancelled, ScanPhase::Cancelled);
    }

    #[test]
    fn test_scan_phase_default() {
        assert_eq!(ScanPhase::default(), ScanPhase::Idle);
    }

    #[test]
    fn test_scan_phase_serde() {
        let phase = ScanPhase::Processing;
        let json = serde_json::to_string(&phase).unwrap();
        assert!(json.contains("processing"));
    }

    #[test]
    fn test_scan_state_default() {
        let state = ScanState::default();
        assert_eq!(state.phase, ScanPhase::Idle);
        assert!(!state.scanning);
        assert_eq!(state.total_files, 0);
        assert_eq!(state.success_count, 0);
        assert_eq!(state.failure_count, 0);
    }

    #[test]
    fn test_scan_state_with_values() {
        let mut state = ScanState::default();
        state.phase = ScanPhase::Processing;
        state.scanning = true;
        state.total_files = 100;
        state.success_count = 50;

        assert_eq!(state.phase, ScanPhase::Processing);
        assert!(state.scanning);
        assert_eq!(state.total_files, 100);
        assert_eq!(state.success_count, 50);
    }

    #[tokio::test]
    async fn test_scan_state_manager_new() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new(tx);

        let state = manager.get_state();
        assert_eq!(state.phase, ScanPhase::Idle);
    }

    #[tokio::test]
    async fn test_scan_state_manager_new_with_interval() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 20);

        let state = manager.get_state();
        assert_eq!(state.phase, ScanPhase::Idle);
    }

    #[tokio::test]
    async fn test_scan_state_manager_set_phase() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.set_phase(ScanPhase::Collecting);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state = manager.get_state();
        assert_eq!(state.phase, ScanPhase::Collecting);
    }

    #[tokio::test]
    async fn test_scan_state_manager_set_total() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.set_total(500);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state = manager.get_state();
        assert_eq!(state.total_files, 500);
    }

    #[tokio::test]
    async fn test_scan_state_manager_increment_success() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.increment_success();
        manager.increment_success();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state = manager.get_state();
        assert_eq!(state.success_count, 2);
    }

    #[tokio::test]
    async fn test_scan_state_manager_increment_failure() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.increment_failure();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state = manager.get_state();
        assert_eq!(state.failure_count, 1);
    }

    #[tokio::test]
    async fn test_scan_state_manager_set_file_counts() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.set_file_counts(10, 5, 3);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state = manager.get_state();
        assert_eq!(state.files_to_add, 10);
        assert_eq!(state.files_to_update, 5);
        assert_eq!(state.files_to_delete, 3);
    }

    #[tokio::test]
    async fn test_scan_state_manager_reset_counters() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.increment_success();
        manager.increment_failure();
        manager.reset_counters();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state = manager.get_state();
        assert_eq!(state.success_count, 0);
        assert_eq!(state.failure_count, 0);
    }

    #[tokio::test]
    async fn test_scan_state_manager_started() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.started();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state = manager.get_state();
        assert!(state.scanning);
        assert!(state.start_time.is_some());
    }

    #[tokio::test]
    async fn test_scan_state_manager_completed() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.completed();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state = manager.get_state();
        // 扫描完成后状态会重置为 Idle，避免 broadcast channel 保存历史消息
        assert!(!state.scanning);
        assert_eq!(state.phase, ScanPhase::Idle);
    }

    #[tokio::test]
    async fn test_scan_state_manager_error() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.error();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state = manager.get_state();
        // 扫描完成后状态会重置为 Idle，避免 broadcast channel 保存历史消息
        assert!(!state.scanning);
        assert_eq!(state.phase, ScanPhase::Idle);
    }

    #[tokio::test]
    async fn test_scan_state_manager_cancelled() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.cancelled();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state = manager.get_state();
        // 扫描完成后状态会重置为 Idle，避免 broadcast channel 保存历史消息
        assert!(!state.scanning);
        assert_eq!(state.phase, ScanPhase::Idle);
    }

    #[tokio::test]
    async fn test_scan_state_manager_get_state() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.set_phase(ScanPhase::Processing);
        manager.set_total(100);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state = manager.get_state();
        assert_eq!(state.phase, ScanPhase::Processing);
        assert_eq!(state.total_files, 100);
    }

    #[tokio::test]
    async fn test_scan_state_manager_to_progress_message() {
        let (tx, _) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        manager.set_phase(ScanPhase::Processing);
        manager.set_total(100);
        manager.increment_success();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let msg = manager.to_progress_message();
        assert_eq!(msg.phase, Some("Processing".to_string()));
        assert_eq!(msg.total_files, 100);
        assert_eq!(msg.success_count, 1);
        assert_eq!(msg.status, "progress");
    }

    #[test]
    fn test_status_from_phase() {
        assert_eq!(ScanStateManager::status_from_phase(&ScanPhase::Idle), "idle");
        assert_eq!(ScanStateManager::status_from_phase(&ScanPhase::Collecting), "progress");
        assert_eq!(ScanStateManager::status_from_phase(&ScanPhase::Counting), "progress");
        assert_eq!(ScanStateManager::status_from_phase(&ScanPhase::Processing), "progress");
        assert_eq!(ScanStateManager::status_from_phase(&ScanPhase::Writing), "progress");
        assert_eq!(ScanStateManager::status_from_phase(&ScanPhase::Deleting), "progress");
        assert_eq!(ScanStateManager::status_from_phase(&ScanPhase::Completed), "completed");
        assert_eq!(ScanStateManager::status_from_phase(&ScanPhase::Error), "error");
        assert_eq!(ScanStateManager::status_from_phase(&ScanPhase::Cancelled), "cancelled");
    }

    #[tokio::test]
    async fn test_scan_state_manager_clone() {
        let (tx, _) = broadcast::channel(100);
        let manager1 = ScanStateManager::new_with_interval(tx, 10);

        manager1.set_phase(ScanPhase::Collecting);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let state1 = manager1.get_state();
        assert_eq!(state1.phase, ScanPhase::Collecting);
    }

    /// 测试扫描完成时会广播消息，然后状态重置为 Idle
    #[tokio::test]
    async fn test_scan_state_manager_broadcast_before_reset() {
        let (tx, mut rx) = broadcast::channel(100);
        let manager = ScanStateManager::new_with_interval(tx, 10);

        // Start a scan
        manager.started();

        // Complete the scan
        manager.completed();

        // Should receive the completed broadcast message (skip the started message)
        let mut completed_msg = None;
        while let Ok(msg) = rx.recv().await {
            if msg.status == "completed" {
                completed_msg = Some(msg);
                break;
            }
        }

        let msg = completed_msg.expect("Should receive completed message");
        assert_eq!(msg.status, "completed");
        assert!(!msg.scanning);

        // Wait for state to reset
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // State should be reset to Idle (not Completed)
        let state = manager.get_state();
        assert_eq!(state.phase, ScanPhase::Idle);
        assert!(!state.scanning);
    }
}
