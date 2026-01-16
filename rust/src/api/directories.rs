use crate::{
    api::AppState,
    app::State,
    db::DirectoryRepository,
};
use axum::{debug_handler, extract::Query, response::IntoResponse, Json};
use serde::Deserialize;

/// Query parameters for directory list
#[derive(Debug, Deserialize)]
pub struct DirectoryQueryParams {
    pub path: Option<String>,
}

#[debug_handler]
pub async fn list_directories(
    State(state): State<AppState>,
    Query(_params): Query<DirectoryQueryParams>,
) -> impl IntoResponse {
    let repo = DirectoryRepository::new(&state.db);

    match repo.find_all().await {
        Ok(directories) => Json(directories).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
