use tokio::sync::{broadcast, mpsc};
use std::sync::{Arc, RwLock};
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
    pub phase_message: String,
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
    SetPhase(ScanPhase, String),
    SetTotal(u64),
    IncrementSuccess,
    IncrementFailure,
    SetFileCounts(u64, u64, u64), // add, update, delete
    Started,
    Completed,
    Error(String),
    Cancelled,
}

/// 扫描状态管理器
pub struct ScanStateManager {
    state: Arc<RwLock<ScanState>>,
    progress_sender: mpsc::Sender<ProgressUpdate>,
    _worker_task: AbortHandle,
}

impl ScanStateManager {
    /// 创建新的状态管理器
    pub fn new(tx: broadcast::Sender<ScanProgressMessage>) -> Self {
        let state = Arc::new(RwLock::new(ScanState::default()));
        let (progress_tx, mut progress_rx) = mpsc::channel(1000);
        let worker_state = state.clone();
        let tx_clone = tx.clone();

        // Worker 任务：接收更新消息，更新状态，广播进度
        let worker_task = tokio::spawn(async move {
            let mut last_progress_reported: u64 = 0;

            while let Some(update) = progress_rx.recv().await {
                {
                    let mut current_state = worker_state.write().unwrap();

                    match update {
                        ProgressUpdate::SetPhase(ref phase, ref message) => {
                            current_state.phase = phase.clone();
                            current_state.phase_message = message.clone();
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
                        ProgressUpdate::Started => {
                            current_state.scanning = true;
                            current_state.start_time = Some(chrono::Utc::now().to_rfc3339());
                        }
                        ProgressUpdate::Completed => {
                            current_state.scanning = false;
                            current_state.phase = ScanPhase::Completed;
                        }
                        ProgressUpdate::Error(ref msg) => {
                            current_state.scanning = false;
                            current_state.phase = ScanPhase::Error;
                            current_state.phase_message = msg.clone();
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

                    // 每 10 个文件发送一次进度消息，或在阶段变更/完成时发送
                    let should_send = matches!(
                        update,
                        ProgressUpdate::SetPhase(_, _)
                            | ProgressUpdate::Started
                            | ProgressUpdate::Completed
                            | ProgressUpdate::Error(_)
                            | ProgressUpdate::Cancelled
                    ) || processed.saturating_sub(last_progress_reported) >= 10;

                    if should_send {
                        let phase_str = format!("{:?}", current_state.phase);
                        let msg = ScanProgressMessage {
                            scanning: current_state.scanning,
                            phase: Some(phase_str),
                            total_files: current_state.total_files,
                            success_count: current_state.success_count,
                            failure_count: current_state.failure_count,
                            progress_percentage: percentage,
                            status: Self::status_from_phase(&current_state.phase),
                            files_to_add: current_state.files_to_add,
                            files_to_update: current_state.files_to_update,
                            files_to_delete: current_state.files_to_delete,
                            start_time: current_state.start_time.clone(),
                        };
                        let _ = tx_clone.send(msg);
                        last_progress_reported = processed;
                    }
                }
            }
        });

        Self {
            state,
            progress_sender: progress_tx,
            _worker_task: worker_task.abort_handle(),
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

    /// 业务逻辑调用的接口
    pub fn set_phase(&self, phase: ScanPhase, message: &str) {
        let _ = self.progress_sender.try_send(ProgressUpdate::SetPhase(phase, message.to_string()));
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

    pub fn started(&self) {
        let _ = self.progress_sender.try_send(ProgressUpdate::Started);
    }

    pub fn completed(&self) {
        let _ = self.progress_sender.try_send(ProgressUpdate::Completed);
    }

    pub fn error(&self, message: &str) {
        let _ = self.progress_sender.try_send(ProgressUpdate::Error(message.to_string()));
    }

    pub fn cancelled(&self) {
        let _ = self.progress_sender.try_send(ProgressUpdate::Cancelled);
    }

    /// 获取当前状态（用于查询）
    pub fn get_state(&self) -> ScanState {
        self.state.read().unwrap().clone()
    }
}
