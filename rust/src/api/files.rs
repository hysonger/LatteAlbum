use crate::{
    api::AppState,
    app::State,
    db::{MediaFile, MediaFileRepository},
};
use axum::{
    debug_handler,
    extract::{Path, Query},
    response::IntoResponse,
    Json,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Query parameters for file list
#[derive(Debug, Deserialize)]
pub struct FileQueryParams {
    pub path: Option<String>,
    pub page: Option<i32>,
    pub size: Option<i32>,
    pub sortBy: Option<String>,
    pub order: Option<String>,
    pub filterType: Option<String>,
    pub cameraModel: Option<String>,
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
    let sort_by = params.sortBy.as_deref().unwrap_or("exifTimestamp");
    let order = params.order.as_deref().unwrap_or("desc");

    let repo = MediaFileRepository::new(&state.db);

    let files = match repo
        .find_all(
            params.path.as_deref(),
            params.filterType.as_deref(),
            params.cameraModel.as_deref(),
            params.date.as_deref(),
            sort_by,
            order,
            page,
            size,
        )
        .await {
        Ok(files) => files,
        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let total = match repo
        .count(params.path.as_deref(), params.filterType.as_deref())
        .await {
        Ok(total) => total,
        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
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
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[debug_handler]
pub async fn get_thumbnail(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(size): Query<ThumbnailSize>,
) -> impl IntoResponse {
    let thumbnail_size = state
        .config
        .get_thumbnail_size(size.size.as_deref().unwrap_or("medium"));

    match state
        .file_service
        .get_thumbnail(&id, thumbnail_size)
        .await
    {
        Ok(Some(data)) => (
            axum::http::StatusCode::OK,
            [("Content-Type", "image/jpeg")],
            data,
        )
            .into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            "Thumbnail not found",
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            e.to_string(),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn get_original(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.file_service.get_original_file(&id).await {
        Ok(Some((data, mime_type))) => (
            axum::http::StatusCode::OK,
            [("Content-Type", mime_type)],
            data,
        )
            .into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            "File not found",
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            e.to_string(),
        )
            .into_response(),
    }
}

#[debug_handler]
pub async fn list_dates(
    State(state): State<AppState>,
    Query(params): Query<FileQueryParams>,
) -> impl IntoResponse {
    let repo = MediaFileRepository::new(&state.db);

    match repo
        .find_dates_with_files(params.path.as_deref(), params.filterType.as_deref())
        .await
    {
        Ok(dates) => Json(dates).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
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
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
