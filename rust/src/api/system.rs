use crate::{api::AppState, app::State};
use axum::{debug_handler, response::IntoResponse, Json};
use serde::Serialize;

/// Response for rescan trigger
#[derive(Debug, Serialize)]
pub struct RescanResponse {
    pub success: bool,
    pub message: String,
}

/// Response for scan progress
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressResponse {
    pub scanning: bool,
    pub phase: Option<String>,
    pub total_files: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub progress_percentage: String,
    pub files_to_add: u64,
    pub files_to_update: u64,
    pub files_to_delete: u64,
    pub start_time: Option<String>,
}

/// Response for cancel operation
#[derive(Debug, Serialize)]
pub struct CancelResponse {
    pub success: bool,
    pub message: String,
}

/// System status response
#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub status: String,
    pub total_files: i64,
    pub image_count: i64,
    pub video_count: i64,
    pub cache_size_mb: f64,
    pub last_scan_time: Option<String>,
}

#[debug_handler]
pub async fn trigger_rescan(State(state): State<AppState>) -> impl IntoResponse {
    let config = &state.config;

    // Start async scan
    let scan_service = state.scan_service.clone();
    let parallel = config.scan_parallel;

    tokio::spawn(async move {
        tracing::info!("Triggering rescan with parallel: {}", parallel);
        scan_service.scan(parallel).await;
    });

    Json(RescanResponse {
        success: true,
        message: "Scan started".to_string(),
    })
}

#[debug_handler]
pub async fn get_scan_progress(State(state): State<AppState>) -> impl IntoResponse {
    let progress = state.broadcaster.get_current_progress().await;

    Json(ScanProgressResponse {
        scanning: progress.scanning,
        phase: progress.phase,
        total_files: progress.total_files,
        success_count: progress.success_count,
        failure_count: progress.failure_count,
        progress_percentage: progress.progress_percentage,
        files_to_add: progress.files_to_add,
        files_to_update: progress.files_to_update,
        files_to_delete: progress.files_to_delete,
        start_time: progress.start_time,
    })
}

#[debug_handler]
pub async fn cancel_scan(State(state): State<AppState>) -> impl IntoResponse {
    let cancelled = state.scan_service.cancel().await;

    Json(CancelResponse {
        success: cancelled,
        message: if cancelled {
            "Scan cancelled".to_string()
        } else {
            "No scan in progress".to_string()
        },
    })
}

#[debug_handler]
pub async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    // Get file counts
    let db = &state.db;
    let total_files = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM media_files")
        .fetch_one(db.get_pool())
        .await
        .unwrap_or(0);

    let image_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM media_files WHERE file_type = 'image'")
        .fetch_one(db.get_pool())
        .await
        .unwrap_or(0);

    let video_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM media_files WHERE file_type = 'video'")
        .fetch_one(db.get_pool())
        .await
        .unwrap_or(0);

    // Calculate cache size
    let cache_size_mb = state
        .cache_service
        .get_cache_size_mb()
        .await
        .unwrap_or(0.0);

    // Get last scan time
    let last_scan_time = sqlx::query_scalar::<_, String>(
        "SELECT MAX(last_scanned) FROM media_files WHERE last_scanned IS NOT NULL"
    )
    .fetch_optional(db.get_pool())
    .await
    .unwrap_or(None);

    Json(SystemStatus {
        status: "running".to_string(),
        total_files,
        image_count,
        video_count,
        cache_size_mb,
        last_scan_time,
    })
}
