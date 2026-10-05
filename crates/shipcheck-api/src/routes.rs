//! Route table and request handlers.

use axum::extract::{DefaultBodyLimit, Path, Query, Request, State};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::model::{Created, NewScan, ProjectSummary, ScanDetail, ScanSummary};
use crate::{auth, db, validate, AppState};

const BODY_LIMIT: usize = 8 * 1024 * 1024;
const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;

pub(crate) fn router(state: AppState) -> Router {
    let guard = middleware::from_fn_with_state(state.clone(), require_key);
    Router::new()
        .route("/health", get(health))
        .route(
            "/api/v1/scans",
            post(create_scan).route_layer(guard).get(list_scans),
        )
        .route("/api/v1/scans/{id}", get(get_scan))
        .route("/api/v1/projects", get(list_projects))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(state)
}

/// Rejects the request before its body is read unless it carries the API key.
async fn require_key(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    auth::authorize(request.headers(), &state.api_key)?;
    Ok(next.run(request).await)
}

async fn health(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    db::ping(&state.pool).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn create_scan(
    State(state): State<AppState>,
    Json(scan): Json<NewScan>,
) -> Result<(StatusCode, Json<Created>), ApiError> {
    validate::new_scan(&scan)?;
    let score = shipcheck_core::score(&scan.findings);
    let id = db::insert_scan(&state.pool, &scan, score).await?;
    let created = Created {
        id,
        score,
        finding_count: scan.findings.len(),
    };
    Ok((StatusCode::CREATED, Json(created)))
}

#[derive(Deserialize)]
struct ListParams {
    project: Option<String>,
    limit: Option<i64>,
}

async fn list_scans(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<ScanSummary>>, ApiError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let scans = db::list_scans(&state.pool, params.project.as_deref(), limit).await?;
    Ok(Json(scans))
}

async fn get_scan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ScanDetail>, ApiError> {
    db::get_scan(&state.pool, id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectSummary>>, ApiError> {
    Ok(Json(db::list_projects(&state.pool).await?))
}
