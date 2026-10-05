//! Errors returned by the HTTP API.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// Everything a request handler can fail with.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// The request body or parameters are invalid.
    #[error("{0}")]
    BadRequest(String),
    /// The API key is missing or wrong.
    #[error("missing or invalid API key")]
    Unauthorized,
    /// The requested record does not exist.
    #[error("not found")]
    NotFound,
    /// The database failed. Details are logged and never sent to clients.
    #[error("internal error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let Self::Database(error) = &self {
            tracing::error!(%error, "database error");
        }
        let status = match &self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(serde_json::json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}
