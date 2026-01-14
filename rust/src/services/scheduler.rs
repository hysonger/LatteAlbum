use crate::services::ScanService;
use std::sync::Arc;
use tracing::info;

/// Scheduler for periodic tasks (simplified)
pub struct Scheduler;

impl Scheduler {
    pub fn new(_scan_service: Arc<ScanService>, _cron_expr: &str) -> Self {
        Self
    }

    /// Start the scheduler
    pub async fn start(&self) {
        info!("Scheduler started (no-op - scheduled scans not implemented)");
    }

    /// Stop the scheduler
    pub async fn stop(&self) {
        info!("Scheduler stopped");
    }
}
