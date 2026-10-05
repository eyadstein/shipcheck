//! Request and response bodies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shipcheck_core::Finding;
use uuid::Uuid;

/// A scan report sent by a client. The score is computed by the server.
#[derive(Debug, Deserialize)]
pub struct NewScan {
    pub project: String,
    #[serde(default)]
    pub git_ref: Option<String>,
    pub findings: Vec<Finding>,
}

/// Answer to a stored scan.
#[derive(Debug, Serialize)]
pub struct Created {
    pub id: Uuid,
    pub score: u32,
    pub finding_count: usize,
}

/// One row of the scan history.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ScanSummary {
    pub id: Uuid,
    pub project: String,
    pub git_ref: Option<String>,
    pub score: i32,
    pub created_at: DateTime<Utc>,
    pub finding_count: i64,
}

/// A stored scan with all of its findings.
#[derive(Debug, Serialize)]
pub struct ScanDetail {
    #[serde(flatten)]
    pub summary: ScanSummary,
    pub findings: Vec<Finding>,
}

/// The latest state of one project.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ProjectSummary {
    pub project: String,
    pub latest_score: i32,
    pub last_scan_at: DateTime<Utc>,
    pub scan_count: i64,
}
