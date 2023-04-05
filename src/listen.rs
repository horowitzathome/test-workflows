use axum::{response::IntoResponse, Json};

pub async fn health() -> impl IntoResponse {
    Json("healthy")
}
