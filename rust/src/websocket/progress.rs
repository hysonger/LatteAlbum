use tokio::sync::{broadcast, mpsc};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use crate::websocket::broadcast::ScanProgressMessage;

/// Unified scan progress tracker that ensures ordered progress updates
pub struct ScanProgressTracker {
    state: Arc<ScanProgressState>,
    result_tx: mpsc::Sender<ProcessingResult>,
    _worker_task: tokio::task::AbortHandle,
}

struct ScanProgressState {
    // 使用 String 而非 Option<String>，确保永不为 None
    phase: Mutex<String>,
    phase_message: Mutex<String>,
    total: AtomicU64,
    success_count: AtomicU64,
    failure_count: AtomicU64,
}

struct ProcessingResult {
    path: PathBuf,
    success: bool,
    error: Option<String>,
}

impl ScanProgressTracker {
    /// 创建新的跟踪器
    pub fn new(tx: broadcast::Sender<ScanProgressMessage>) -> Self {
        let state = Arc::new(ScanProgressState {
            phase: Mutex::new(String::new()),
            phase_message: Mutex::new(String::new()),
            total: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            failure_count: AtomicU64::new(0),
        });

        let (result_tx, mut result_rx) = mpsc::channel::<ProcessingResult>(1000);
        let worker_state = state.clone();
        let tx_clone = tx.clone();

        // 专用任务按顺序处理结果并发送进度
        let worker_task = tokio::spawn(async move {
            let mut processed: u64 = 0;

            while let Some(result) = result_rx.recv().await {
                // 更新计数
                if result.success {
                    worker_state.success_count.fetch_add(1, Ordering::SeqCst);
                } else {
                    worker_state.failure_count.fetch_add(1, Ordering::SeqCst);
                }
                processed += 1;

                // 获取当前状态（包含 phase）
                let phase = worker_state.phase.lock().unwrap().clone();
                let phase_message = worker_state.phase_message.lock().unwrap().clone();
                let total = worker_state.total.load(Ordering::SeqCst);
                let success = worker_state.success_count.load(Ordering::SeqCst);
                let failure = worker_state.failure_count.load(Ordering::SeqCst);

                // 计算进度百分比
                let percentage = if total > 0 {
                    processed as f64 / total as f64 * 100.0
                } else {
                    0.0
                };

                // 每 5 个文件或结束时发送进度
                if processed % 5 == 0 || processed == total {
                    let msg = ScanProgressMessage {
                        scanning: true,
                        status: "progress".to_string(),
                        phase: Some(phase),
                        phase_message: Some(phase_message),
                        total_files: total,
                        success_count: success,
                        failure_count: failure,
                        progress_percentage: format!("{:.2}", percentage),
                        files_to_add: 0,
                        files_to_update: 0,
                        files_to_delete: 0,
                        start_time: None,
                    };
                    let _ = tx_clone.send(msg);
                }
            }
        });

        Self {
            state,
            result_tx,
            _worker_task: worker_task.abort_handle(),
        }
    }

    /// 设置当前阶段（必须调用，确保 phase 不为 None）
    pub fn set_phase(&self, phase: &str, message: &str) {
        let mut p = self.state.phase.lock().unwrap();
        let mut m = self.state.phase_message.lock().unwrap();
        *p = phase.to_string();
        *m = message.to_string();
    }

    /// 设置总数
    pub fn set_total(&self, total: u64) {
        self.state.total.store(total, Ordering::SeqCst);
    }

    /// 报告处理结果（线程安全）
    pub async fn report_result(&self, path: PathBuf, success: bool, error: Option<String>) {
        let result = ProcessingResult { path, success, error };
        let _ = self.result_tx.send(result).await;
    }

    /// 获取当前计数（用于写入阶段）
    pub fn get_counts(&self) -> (u64, u64) {
        let success = self.state.success_count.load(Ordering::SeqCst);
        let failure = self.state.failure_count.load(Ordering::SeqCst);
        (success, failure)
    }

    /// 获取总数
    pub fn get_total(&self) -> u64 {
        self.state.total.load(Ordering::SeqCst)
    }

    /// 获取当前 phase（用于发送消息时）
    pub fn get_phase_info(&self) -> (String, String) {
        let phase = self.state.phase.lock().unwrap().clone();
        let phase_message = self.state.phase_message.lock().unwrap().clone();
        (phase, phase_message)
    }
}
