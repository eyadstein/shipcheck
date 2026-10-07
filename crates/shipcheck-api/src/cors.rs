//! Cross-origin access for browser dashboards.

use axum::http::{HeaderValue, Method};
use axum::Router;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Lets the listed browser origins read the API.
///
/// Only `GET` is allowed, so the API key never has to travel through a browser.
/// An empty list leaves the router unchanged.
pub fn with_cors(router: Router, origins: &[String]) -> Router {
    let allowed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();
    if allowed.is_empty() {
        return router;
    }
    let layer = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed))
        .allow_methods([Method::GET]);
    router.layer(layer)
}
