use crate::{
    api::AppState,
    app::State,
    db::{MediaFile, MediaFileRepository},
};
use axum::{
    body::Body,
    debug_handler,
    extract::{Path, Query},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tracing::warn;
use tokio_util::io::ReaderStream;

/// Get size label from size string
/// This is used to determine the cache key and which thumbnail size to generate
fn get_size_label(size_str: &str) -> &'static str {
    match size_str {
        "small" => "small",
        "medium" => "medium",
        "large" => "large",
        "full" => "full",
        _ => "medium", // default
    }
}

/// Query parameters for file list
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileQueryParams {
    pub path: Option<String>,
    pub page: Option<i32>,
    pub size: Option<i32>,
    #[serde(rename = "sortBy")]
    pub sort_by: Option<String>,
    pub order: Option<String>,
    #[serde(rename = "filterType")]
    pub filter_type: Option<String>,
    #[serde(rename = "cameraModel")]
    pub camera_model: Option<String>,
    pub date: Option<String>,
}

/// Pagination response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub size: i32,
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
}

/// Date with count response
#[derive(Debug, Serialize)]
pub struct DateResponse {
    pub date: String,
    pub count: i64,
}

/// Neighbor response for navigation
#[derive(Debug, Serialize)]
pub struct NeighborResponse {
    pub previous: Option<MediaFile>,
    pub next: Option<MediaFile>,
}

/// Thumbnail size enum
#[derive(Debug, Deserialize)]
pub struct ThumbnailSize {
    pub size: Option<String>,
}

#[debug_handler]
pub async fn list_files(
    State(state): State<AppState>,
    Query(params): Query<FileQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(0);
    let size = params.size.unwrap_or(50);
    let sort_by = params.sort_by.as_deref().unwrap_or("exifTimestamp");
    let order = params.order.as_deref().unwrap_or("desc");

    let repo = MediaFileRepository::new(&state.db);

    let files = match repo
        .find_all(
            params.path.as_deref(),
            params.filter_type.as_deref(),
            params.camera_model.as_deref(),
            params.date.as_deref(),
            sort_by,
            order,
            page,
            size,
        )
        .await {
        Ok(files) => files,
        Err(e) => {
            warn!("Failed to query files: {}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let total = match repo
        .count(params.path.as_deref(), params.filter_type.as_deref())
        .await {
        Ok(total) => total,
        Err(e) => {
            warn!("Failed to count files: {}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let total_pages = ((total as f64) / (size as f64)).ceil() as i32;

    Json(PaginatedResponse {
        items: files,
        total,
        page,
        size,
        total_pages,
    }).into_response()
}

#[debug_handler]
pub async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let repo = MediaFileRepository::new(&state.db);

    match repo.find_by_id(&id).await {
        Ok(Some(file)) => Json(file).into_response(),
        Ok(None) => (axum::http::StatusCode::NOT_FOUND, "File not found").into_response(),
        Err(e) => {
            warn!("Failed to get file {}: {}", id, e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

#[debug_handler]
pub async fn get_thumbnail(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(size): Query<ThumbnailSize>,
) -> impl IntoResponse {
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::response::Response;
    use std::fmt::Write;
    use tokio::fs::File;
    use tokio_util::io::ReaderStream;

    let size_str = size.size.as_deref().unwrap_or("medium");
    let thumbnail_size = state.config.get_thumbnail_size(size_str);
    let fit_to_height = size_str == "large";  // large size uses fixed height
    let size_label = get_size_label(size_str);

    // 1. Check memory cache first - return directly if hit (already in memory)
    if let Some(data) = state.cache_service.get_thumbnail(&id, &size_label).await {
        let mut etag = String::with_capacity(64);
        write!(&mut etag, "\"{}-{}}}\"", id, size_label).unwrap();

        let mut response = Response::new(Body::from(data));
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("image/jpeg"),
        );
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("public, max-age=86400"),
        );
        response.headers_mut().insert(
            axum::http::header::ETAG,
            axum::http::HeaderValue::from_str(&etag).unwrap(),
        );
        return response;
    }

    // 2. Check disk cache - stream from file if exists
    if let Some(disk_path) = state.cache_service.get_thumbnail_disk_path(&id, &size_label) {
        match File::open(&disk_path).await {
            Ok(file) => {
                let file_size = tokio::fs::metadata(&disk_path).await.map(|m| m.len()).unwrap_or(0);

                let mut etag = String::with_capacity(64);
                write!(&mut etag, "\"{}-{}}}\"", id, size_label).unwrap();

                let stream = ReaderStream::with_capacity(file, 32 * 1024);

                let mut response_headers = HeaderMap::new();
                response_headers.insert(
                    axum::http::header::CONTENT_TYPE,
                    axum::http::HeaderValue::from_static("image/jpeg"),
                );
                response_headers.insert(
                    axum::http::header::CONTENT_LENGTH,
                    file_size.to_string().parse().unwrap(),
                );
                response_headers.insert(
                    axum::http::header::CACHE_CONTROL,
                    axum::http::HeaderValue::from_static("public, max-age=86400"),
                );
                response_headers.insert(
                    axum::http::header::ETAG,
                    axum::http::HeaderValue::from_str(&etag).unwrap(),
                );

                return (StatusCode::OK, response_headers, Body::from_stream(stream)).into_response();
            }
            Err(e) => {
                tracing::warn!("Failed to open disk cache file {}: {}", disk_path.display(), e);
                // Continue to generate new thumbnail
            }
        }
    }

    // 3. Not in cache - generate thumbnail
    match state.file_service.get_thumbnail(&id, &size_label, thumbnail_size, fit_to_height).await {
        Ok(Some((data, mime_type))) => {
            let mut etag = String::with_capacity(64);
            write!(&mut etag, "\"{}-{}}}\"", id, size_label).unwrap();

            let mut response = Response::new(Body::from(data));
            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_str(&mime_type).unwrap_or_else(|_| {
                    axum::http::HeaderValue::from_static("image/jpeg")
                }),
            );
            response.headers_mut().insert(
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static("public, max-age=86400"),
            );
            response.headers_mut().insert(
                axum::http::header::ETAG,
                axum::http::HeaderValue::from_str(&etag).unwrap(),
            );
            response
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Thumbnail not found").into_response(),
        Err(e) => {
            warn!("Failed to get thumbnail for {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

#[debug_handler]
pub async fn get_original(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    use axum::http::StatusCode;
    use std::io::SeekFrom;
    use tokio::io::AsyncSeekExt;

    let repo = MediaFileRepository::new(&state.db);

    match repo.find_by_id(&id).await {
        Ok(Some(file)) => {
            let path = std::path::Path::new(&file.file_path);
            if !path.exists() {
                return (StatusCode::NOT_FOUND, "File not found").into_response();
            }

            let mime_type = file.mime_type.unwrap_or_else(|| {
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();
                match ext.as_str() {
                    "mp4" => "video/mp4".to_string(),
                    "mov" => "video/quicktime".to_string(),
                    "avi" => "video/x-msvideo".to_string(),
                    "mkv" => "video/x-matroska".to_string(),
                    "webm" => "video/webm".to_string(),
                    "jpg" | "jpeg" => "image/jpeg".to_string(),
                    "png" => "image/png".to_string(),
                    _ => "application/octet-stream".to_string(),
                }
            });

            let file_size = tokio::fs::metadata(path).await
                .map(|m| m.len())
                .unwrap_or(0);

            if file_size == 0 {
                return (StatusCode::NOT_FOUND, "Empty file").into_response();
            }

            // Check for Range header (video streaming)
            let range_header = headers.get("range");

            if let Some(range_value) = range_header {
                // Parse Range header: "bytes=start-end"
                let range_str = range_value.to_str().unwrap_or("");
                if range_str.starts_with("bytes=") {
                    let ranges: Vec<&str> = range_str[6..].split(',').collect();
                    if let Some(range_part) = ranges.first() {
                        let parts: Vec<&str> = range_part.trim().split('-').collect();
                        if parts.len() == 2 {
                            let start: u64 = parts[0].parse().unwrap_or(0);
                            let end: u64 = parts[1].parse().unwrap_or(file_size.saturating_sub(1));

                            // Clamp to file size
                            let start = start.min(file_size.saturating_sub(1));
                            let end = end.min(file_size.saturating_sub(1));
                            if start > end {
                                return (StatusCode::RANGE_NOT_SATISFIABLE, "Invalid range").into_response();
                            }

                            let content_length: u64 = end.saturating_sub(start).saturating_add(1);

                            // Open file and seek to start position
                            let mut file = match File::open(path).await {
                                Ok(f) => f,
                                Err(e) => {
                                    warn!("Failed to open file {}: {}", path.display(), e);
                                    return (StatusCode::NOT_FOUND, "Cannot open file").into_response();
                                }
                            };

                            if start > 0 {
                                if let Err(e) = file.seek(SeekFrom::Start(start)).await {
                                    warn!("Failed to seek in file {}: {}", path.display(), e);
                                    return (StatusCode::INTERNAL_SERVER_ERROR, "Seek failed").into_response();
                                }
                            }

                            // Create streaming response
                            let stream = ReaderStream::with_capacity(file, 64 * 1024);

                            let mut response_headers = HeaderMap::new();
                            response_headers.insert("Content-Type", mime_type.parse().unwrap());
                            response_headers.insert("Content-Length", content_length.to_string().parse().unwrap());
                            response_headers.insert("Content-Range", format!("bytes {}-{}/{}", start, end, file_size).parse().unwrap());
                            response_headers.insert("Accept-Ranges", "bytes".parse().unwrap());

                            return (StatusCode::PARTIAL_CONTENT, response_headers, Body::from_stream(stream)).into_response();
                        }
                    }
                }
            }

            // Full file request - use streaming for large files (videos)
            // For images under 50MB, load into memory; for videos, always stream
            if file_size > 50 * 1024 * 1024 {
                // Large file (video) - stream it
                let file = match File::open(path).await {
                    Ok(f) => f,
                    Err(e) => {
                        warn!("Failed to open large file {}: {}", path.display(), e);
                        return (StatusCode::NOT_FOUND, "Cannot open file").into_response();
                    }
                };
                let stream = ReaderStream::with_capacity(file, 64 * 1024 * 1024);

                let mut headers = HeaderMap::new();
                headers.insert("Content-Type", mime_type.parse().unwrap());
                headers.insert("Content-Length", file_size.to_string().parse().unwrap());
                headers.insert("Accept-Ranges", "bytes".parse().unwrap());

                (StatusCode::OK, headers, Body::from_stream(stream)).into_response()
            } else {
                // Small file - read into memory
                match tokio::fs::read(path).await {
                    Ok(data) => {
                        let mut headers = HeaderMap::new();
                        headers.insert("Content-Type", mime_type.parse().unwrap());
                        headers.insert("Content-Length", data.len().to_string().parse().unwrap());
                        headers.insert("Accept-Ranges", "bytes".parse().unwrap());

                        (StatusCode::OK, headers, data).into_response()
                    }
                    Err(e) => {
                        warn!("Failed to read file {}: {}", path.display(), e);
                        (StatusCode::NOT_FOUND, "Cannot read file").into_response()
                    }
                }
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "File not found").into_response(),
        Err(e) => {
            warn!("Failed to get original file {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

#[debug_handler]
pub async fn list_dates(
    State(state): State<AppState>,
    Query(params): Query<FileQueryParams>,
) -> impl IntoResponse {
    let repo = MediaFileRepository::new(&state.db);

    match repo
        .find_dates_with_files(params.path.as_deref(), params.filter_type.as_deref())
        .await
    {
        Ok(dates) => Json(dates).into_response(),
        Err(e) => {
            warn!("Failed to query dates: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

#[debug_handler]
pub async fn get_neighbors(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let repo = MediaFileRepository::new(&state.db);

    match repo.find_by_id(&id).await {
        Ok(Some(file)) => {
            let response = if let Some(sort_time) = file.get_effective_sort_time() {
                let previous = repo.find_neighbors(&id, sort_time, true).await.unwrap_or(None);
                let next = repo.find_neighbors(&id, sort_time, false).await.unwrap_or(None);

                NeighborResponse { previous, next }
            } else {
                NeighborResponse {
                    previous: None,
                    next: None,
                }
            };
            Json(response).into_response()
        }
        Ok(None) => (axum::http::StatusCode::NOT_FOUND, "File not found").into_response(),
        Err(e) => {
            warn!("Failed to get neighbors for {}: {}", id, e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
