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
use std::path::PathBuf;
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
    /// Canonicalized absolute path to the assets directory.
    /// Pre-computed once at startup to avoid repeated canonicalization
    /// and used for path traversal prevention.
    /// `None` when the assets directory does not exist (e.g. tests,
    /// frontend not built yet). In that case all static requests get 404.
    pub assets_base_path: Option<PathBuf>,
}

/// Main application structure
pub struct App {
    state: AppState,
    router: Router,
}

impl App {
    /// Get a clone of the router for testing
    pub fn router_clone(&self) -> Router {
        self.router.clone()
    }

    /// Create a new application instance
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize database
        let db = DatabasePool::new(&config.db_path).await?;

        // Run migrations
        let migrations_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/db/migrations");
        db.migrate(&migrations_path).await?;
        tracing::info!(
            "Database migrations applied. GPS columns (gps_latitude, gps_longitude) available. \
             Run a full rescan to populate GPS data for existing photos."
        );

        // Create cache directory
        tokio::fs::create_dir_all(&config.cache_dir).await?;

        // Create shared state
        let mut broadcaster = Arc::new(ScanProgressBroadcaster::new());
        let scan_state = Arc::new(ScanStateManager::new_with_interval(
            broadcaster.sender(),
            config.ws_progress_broadcast_interval,
        ));

        // Set scan_state reference in broadcaster (break circular dependency)
        Arc::make_mut(&mut broadcaster).set_scan_state(scan_state.clone());

        // Create cache service with configurable parameters
        let cache_service = Arc::new(CacheService::new(
            &config.cache_dir,
            config.cache_max_capacity,
            config.cache_ttl_seconds,
        ).await?);

        // Create transcoding pool for CPU-intensive image processing (MUST be created before processors)
        let transcoding_pool = Arc::new(TranscodingPool::new(config.transcoding_threads));

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
            &config,
        ));

        // Compute the canonicalized assets base path once at startup.
        // This serves two purposes:
        // 1. Performance: avoids repeated canonicalization on every static file request.
        // 2. Security: the canonicalized path is used as a prefix anchor for path
        //    traversal checks in serve_static.
        //
        // If the assets directory does not exist (e.g. in tests, or when the
        // frontend has not been built yet), `assets_base_path` will be `None`,
        // and all static-file requests will receive 404.
        let static_assets_path = config.static_dir.join("assets");
        let assets_base_path = std::fs::canonicalize(&static_assets_path).ok();

        let state = AppState {
            config,
            db,
            file_service,
            scan_service,
            cache_service,
            broadcaster,
            scan_state,
            processors,
            assets_base_path,
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
            .route("/api/files/{id}/gps", get(files::get_file_gps))
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

    /// Serve static assets with path traversal protection.
    ///
    /// The handler does the following for every request:
    /// 1. Validates the user-supplied path against common bypass techniques
    ///    (empty, null bytes).
    /// 2. Joins the path to the pre-canonicalized assets base path.
    /// 3. Canonicalizes the resulting path (resolves `..`, symlinks, etc.).
    /// 4. Verifies that the canonicalized result is still within the assets
    ///    base path.
    /// 5. Ensures only regular files are served (not directories or symlinks
    ///    pointing outside the tree).
    ///
    /// If at any step the path escapes or the file type is wrong, the request
    /// is rejected (403 Forbidden for traversal, 404 Not Found for missing
    /// files).
    async fn serve_static(
        State(state): State<AppState>,
        Path(path): Path<String>,
    ) -> impl IntoResponse {
        // If the assets directory doesn't exist (not built yet, tests, ...),
        // every static-file request is a 404.
        let assets_base = match &state.assets_base_path {
            Some(p) => p,
            None => {
                return (axum::http::StatusCode::NOT_FOUND, "Not found").into_response();
            }
        };

        // 1. Reject empty paths
        if path.trim().is_empty() {
            return (axum::http::StatusCode::BAD_REQUEST, "Empty path").into_response();
        }

        // 2. Reject paths that contain a null byte — these would be truncated by
        //    the OS (e.g. "foo\0bar" → "foo"), allowing bypass of the prefix check.
        if path.contains('\0') {
            return (axum::http::StatusCode::BAD_REQUEST, "Invalid path").into_response();
        }

        // 3. Join the user path onto the trusted base and canonicalize.
        //    Path::join handles `.` and `..` components naively — canonicalize
        //    resolves these as well as symlinks into an absolute, normalized path.
        let file_path = assets_base.join(&path);

        let resolved = match std::fs::canonicalize(&file_path) {
            Ok(p) => p,
            Err(_) => {
                return (axum::http::StatusCode::NOT_FOUND, "Not found").into_response();
            }
        };

        // 4. Path traversal check: the resolved path MUST start with the
        //    pre-canonicalized assets base path. This is the core security check.
        if !resolved.starts_with(assets_base) {
            tracing::warn!(
                "Path traversal attempt blocked: requested={} resolved={}",
                path,
                resolved.display()
            );
            return (
                axum::http::StatusCode::FORBIDDEN,
                "Access denied",
            )
                .into_response();
        }

        // 5. Only serve regular files (not directories, not symlinks to outside).
        //    canonicalize already resolved symlinks; an explicit is_file check is
        //    defense-in-depth.
        match tokio::fs::metadata(&resolved).await {
            Ok(meta) if meta.is_file() => {}
            Ok(_) => {
                return (axum::http::StatusCode::NOT_FOUND, "Not found").into_response();
            }
            Err(_) => {
                return (axum::http::StatusCode::NOT_FOUND, "Not found").into_response();
            }
        }

        // 6. Read and serve the file
        match tokio::fs::read(&resolved).await {
            Ok(content) => {
                let mime_type = mime_guess::from_path(&resolved)
                    .first()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                Response::builder()
                    .header("Content-Type", mime_type)
                    .body(Body::from(content))
                    .unwrap()
            }
            Err(_) => (axum::http::StatusCode::NOT_FOUND, "Not found").into_response(),
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
            // Spawn initial scan in background
            let scan_service = self.state.scan_service.clone();
            tokio::spawn(async move {
                scan_service.scan().await;
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
