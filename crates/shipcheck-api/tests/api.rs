use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use shipcheck_api::{build_router, db, AppState};
use tower::ServiceExt;
use uuid::Uuid;

const KEY: &str = "test-key-0123456789";

/// A router whose pool never connects. Enough for tests that stop before the database.
fn lazy_app() -> Router {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://nobody:none@127.0.0.1:1/none")
        .expect("lazy pool");
    build_router(AppState::new(pool, KEY))
}

/// A router backed by the real database, or `None` when `DATABASE_URL` is not set.
async fn db_app() -> Option<Router> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = db::connect(&url).await.expect("database is reachable");
    db::migrate(&pool).await.expect("migrations apply");
    Some(build_router(AppState::new(pool, KEY)))
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn post_scan(body: &Value, key: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/v1/scans")
        .header("content-type", "application/json");
    if let Some(key) = key {
        builder = builder.header("x-api-key", key);
    }
    builder.body(Body::from(body.to_string())).expect("request")
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

fn finding(severity: &str) -> Value {
    json!({
        "rule_id": "SEC-101",
        "category": "security",
        "severity": severity,
        "message": "message",
        "file": "a.js",
        "line": 3,
        "fix": null
    })
}

fn unique_project() -> String {
    format!("test/{}", Uuid::new_v4())
}

#[tokio::test]
async fn writes_require_the_api_key() {
    let app = lazy_app();
    let body = json!({ "project": "a/b", "findings": [] });
    let (missing, _) = send(&app, post_scan(&body, None)).await;
    assert_eq!(missing, StatusCode::UNAUTHORIZED);
    let (wrong, _) = send(&app, post_scan(&body, Some("not-the-key"))).await;
    assert_eq!(wrong, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn invalid_project_names_are_rejected() {
    let app = lazy_app();
    let body = json!({ "project": "has spaces", "findings": [] });
    let (status, answer) = send(&app, post_scan(&body, Some(KEY))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(answer["error"]
        .as_str()
        .unwrap_or_default()
        .contains("project"));
}

#[tokio::test]
async fn unknown_severities_are_rejected() {
    let app = lazy_app();
    let body = json!({ "project": "a/b", "findings": [finding("catastrophic")] });
    let (status, _) = send(&app, post_scan(&body, Some(KEY))).await;
    assert!(status.is_client_error());
}

#[tokio::test]
async fn health_reports_ok() {
    let Some(app) = db_app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let (status, body) = send(&app, get("/health")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn stored_scan_can_be_fetched_with_server_side_score() {
    let Some(app) = db_app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let project = unique_project();
    let body = json!({
        "project": project,
        "git_ref": "main",
        "score": 100,
        "findings": [finding("high"), finding("low")]
    });
    let (status, created) = send(&app, post_scan(&body, Some(KEY))).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["score"], 93);
    let id = created["id"].as_str().expect("id").to_owned();

    let (status, detail) = send(&app, get(&format!("/api/v1/scans/{id}"))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["project"], project.as_str());
    assert_eq!(detail["score"], 93);
    assert_eq!(detail["finding_count"], 2);
    assert_eq!(detail["findings"].as_array().expect("findings").len(), 2);
    assert_eq!(detail["findings"][0]["severity"], "high");
}

#[tokio::test]
async fn history_can_be_filtered_and_limited() {
    let Some(app) = db_app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let project = unique_project();
    let body = json!({ "project": project, "findings": [] });
    for _ in 0..2 {
        let (status, _) = send(&app, post_scan(&body, Some(KEY))).await;
        assert_eq!(status, StatusCode::CREATED);
    }
    let (_, all) = send(&app, get(&format!("/api/v1/scans?project={project}"))).await;
    assert_eq!(all.as_array().expect("list").len(), 2);
    let (_, one) = send(
        &app,
        get(&format!("/api/v1/scans?project={project}&limit=1")),
    )
    .await;
    assert_eq!(one.as_array().expect("list").len(), 1);
}

#[tokio::test]
async fn projects_show_their_latest_score() {
    let Some(app) = db_app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let project = unique_project();
    let worse = json!({ "project": project, "findings": [finding("critical")] });
    let better = json!({ "project": project, "findings": [] });
    send(&app, post_scan(&worse, Some(KEY))).await;
    send(&app, post_scan(&better, Some(KEY))).await;
    let (_, projects) = send(&app, get("/api/v1/projects")).await;
    let entry = projects
        .as_array()
        .expect("list")
        .iter()
        .find(|item| item["project"] == project.as_str())
        .expect("project is listed");
    assert_eq!(entry["latest_score"], 100);
    assert_eq!(entry["scan_count"], 2);
}

#[tokio::test]
async fn unknown_scan_is_not_found() {
    let Some(app) = db_app().await else {
        eprintln!("skipping: DATABASE_URL is not set");
        return;
    };
    let (status, _) = send(&app, get(&format!("/api/v1/scans/{}", Uuid::new_v4()))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

async fn preflight(app: &Router, origin: &str) -> Option<String> {
    let request = Request::builder()
        .method("OPTIONS")
        .uri("/api/v1/scans")
        .header("origin", origin)
        .header("access-control-request-method", "GET")
        .body(Body::empty())
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    response
        .headers()
        .get("access-control-allow-origin")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

#[tokio::test]
async fn listed_origins_may_read_and_others_may_not() {
    let app = shipcheck_api::with_cors(lazy_app(), &["https://dashboard.example".to_owned()]);
    assert_eq!(
        preflight(&app, "https://dashboard.example")
            .await
            .as_deref(),
        Some("https://dashboard.example")
    );
    assert_eq!(preflight(&app, "https://evil.example").await, None);
}

#[tokio::test]
async fn no_origins_means_no_cors_headers() {
    let app = shipcheck_api::with_cors(lazy_app(), &[]);
    assert_eq!(preflight(&app, "https://dashboard.example").await, None);
}
