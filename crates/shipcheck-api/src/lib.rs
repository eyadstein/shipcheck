//! HTTP API that stores Shipcheck scan reports in PostgreSQL.

// Axum extractors must be taken by value, and descriptive type names read better
// than clippy's module repetition rule.
#![allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]

mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod model;
mod routes;
mod validate;

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

/// Shared state handed to every request handler.
#[derive(Clone)]
pub struct AppState {
    pub(crate) pool: PgPool,
    pub(crate) api_key: Arc<str>,
}

impl AppState {
    /// Creates the state from a connection pool and the key that guards writes.
    #[must_use]
    pub fn new(pool: PgPool, api_key: &str) -> Self {
        Self {
            pool,
            api_key: Arc::from(api_key),
        }
    }
}

/// Builds the router with every route and the write guard attached.
pub fn build_router(state: AppState) -> Router {
    routes::router(state)
}
