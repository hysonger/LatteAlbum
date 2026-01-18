use crate::api::{files, directories, system};
use crate::config::Config;
use crate::db::{DatabasePool, MediaFileRepository};
use crate::processors::{ProcessorRegistry, image_processor::StandardImageProcessor, heif_processor::HeifImageProcessor, video_processor::VideoProcessor};
use crate::services::{FileService, ScanService, CacheService, Scheduler, TranscodingPool};
use crate::websocket::{ScanProgressBroadcaster, ScanStateManager};
use axum::{
    body::Body,
    extract::Path,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: DatabasePool,
    pub file_service: Arc<FileService>,
    pub scan_service: Arc<ScanService>,
    pub cache_service: Arc<CacheService>,
    pub broadcaster: Arc<ScanProgressBroadcaster>,
    pub scan_state: Arc<ScanStateManager>,
    pub processors: Arc<ProcessorRegistry>,
}

/// Main application structure
pub struct App {
    state: AppState,
    router: Router,
}

impl App {
    /// Create a new application instance
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize database
        let db = DatabasePool::new(&config.db_path).await?;

        // Run migrations
        let migrations_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/db/migrations");
        db.migrate(&migrations_path).await?;

        // Create cache directory
        tokio::fs::create_dir_all(&config.cache_dir).await?;

        // Create shared state
        let mut broadcaster = Arc::new(ScanProgressBroadcaster::new());
        let scan_state = Arc::new(ScanStateManager::new(broadcaster.sender()));

        // Set scan_state reference in broadcaster (break circular dependency)
        Arc::make_mut(&mut broadcaster).set_scan_state(scan_state.clone());

        let cache_service = Arc::new(CacheService::new(&config.cache_dir).await?);

        // Create transcoding pool for CPU-intensive image processing (MUST be created before processors)
        let transcoding_pool = Arc::new(TranscodingPool::new(4));

        // Initialize processor registry with transcoding pool
        let mut processors = ProcessorRegistry::new(Some(transcoding_pool.clone()));

        processors.register(Arc::new(HeifImageProcessor::new(Some(transcoding_pool.clone()))));
        processors.register(Arc::new(StandardImageProcessor::new()));
        processors.register(Arc::new(VideoProcessor::new(Some(config.ffmpeg_path.to_string_lossy().to_string()))));
        let processors = Arc::new(processors);

        let scan_service = Arc::new(ScanService::new(
            config.clone(),
            db.clone(),
            processors.clone(),
            scan_state.clone(),
        ));

        let file_service = Arc::new(FileService::new(
            db.clone(),
            cache_service.clone(),
            processors.clone(),
            transcoding_pool.clone(),
        ));

        let state = AppState {
            config,
            db,
            file_service,
            scan_service,
            cache_service,
            broadcaster,
            scan_state,
            processors,
        };

        // Build router
        let router = Self::build_router(&state);

        Ok(Self { state, router })
    }

    /// Build the application router
    fn build_router(state: &AppState) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::DELETE])
            .allow_headers(Any);

        Router::new()
            .route("/", get(Self::serve_index))
            .route("/assets/{*path}", get(Self::serve_static))
            .route("/api/files", get(files::list_files))
            .route("/api/files/dates", get(files::list_dates))
            .route("/api/files/{id}", get(files::get_file))
            .route("/api/files/{id}/thumbnail", get(files::get_thumbnail))
            .route("/api/files/{id}/original", get(files::get_original))
            .route("/api/files/{id}/neighbors", get(files::get_neighbors))
            .route("/api/directories", get(directories::list_directories))
            .route("/api/system/rescan", post(system::trigger_rescan))
            .route("/api/system/scan/progress", get(system::get_scan_progress))
            .route("/api/system/scan/cancel", post(system::cancel_scan))
            .route("/api/system/status", get(system::get_status))
            .route("/ws/scan", get(Self::websocket_handler))
            .layer(cors)
            .with_state(state.clone())
    }

    /// Serve index.html
    async fn serve_index() -> impl IntoResponse {
        let static_dir = std::env::var("LATTE_STATIC_DIR")
            .unwrap_or_else(|_| "./static/dist".to_string());

        let index_path = std::path::PathBuf::from(&static_dir).join("index.html");

        match tokio::fs::read_to_string(&index_path).await {
            Ok(content) => Html(content),
            Err(_) => Html("<html><body><h1>Latte Album</h1><p>Frontend not found. Please build the frontend first.</p></body></html>".to_string()),
        }
    }

    /// Serve static assets
    async fn serve_static(Path(path): Path<String>) -> impl IntoResponse {
        let static_dir = std::env::var("LATTE_STATIC_DIR")
            .unwrap_or_else(|_| "./static/dist".to_string());

        let static_path = std::path::PathBuf::from(&static_dir).join("assets");
        let file_path = static_path.join(&path);

        if let Ok(content) = tokio::fs::read(&file_path).await {
            let mime_type = mime_guess::from_path(&file_path)
                .first()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());

            Response::builder()
                .header("Content-Type", mime_type)
                .body(Body::from(content))
                .unwrap()
        } else {
            (axum::http::StatusCode::NOT_FOUND, "Not found").into_response()
        }
    }

    /// WebSocket handler
    async fn websocket_handler(
        State(state): State<AppState>,
        ws: axum::extract::ws::WebSocketUpgrade,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| {
            crate::websocket::handle_websocket(socket, state.broadcaster.clone())
        })
    }

    /// Run the application
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.state.config.host, self.state.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Server listening on {}", addr);

        // Check if first run (database empty) and trigger initial scan
        let repo = MediaFileRepository::new(&self.state.db);
        if repo.is_empty().await? {
            info!("First run detected - starting initial scan...");
            // Spawn scan in blocking thread pool to avoid blocking API requests
            let scan_service = self.state.scan_service.clone();
            tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    scan_service.scan(true).await;
                });
            });
        }

        // Start scheduler
        let scheduler = Scheduler::new(
            self.state.scan_service.clone(),
            &self.state.config.scan_cron,
        );
        scheduler.start().await;

        axum::serve(listener, self.router).await?;
        Ok(())
    }
}

// Re-export State extractor for use in handlers
pub use axum::extract::State;
