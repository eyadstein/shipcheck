//! Database access. Every query is parameterized.

use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;
use shipcheck_core::Finding;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::model::{NewScan, ProjectSummary, ScanDetail, ScanSummary};

const SUMMARY: &str = "\
    SELECT s.id, s.project, s.git_ref, s.score, s.created_at, \
    (SELECT count(*) FROM findings f WHERE f.scan_id = s.id) AS finding_count \
    FROM scans s";

const INSERT_FINDINGS: &str = "\
    INSERT INTO findings (scan_id, rule_id, category, severity, message, file, line, fix) \
    SELECT $1::uuid, t.rule_id, t.category, t.severity, t.message, t.file, t.line, t.fix \
    FROM UNNEST($2::text[], $3::text[], $4::text[], $5::text[], $6::text[], $7::int[], $8::text[]) \
    AS t(rule_id, category, severity, message, file, line, fix)";

const SELECT_FINDINGS: &str = "\
    SELECT rule_id, category, severity, message, file, line, fix \
    FROM findings WHERE scan_id = $1 ORDER BY id";

const LATEST_PER_PROJECT: &str = "\
    SELECT DISTINCT ON (project) project, score AS latest_score, \
    created_at AS last_scan_at, \
    (SELECT count(*) FROM scans c WHERE c.project = scans.project) AS scan_count \
    FROM scans ORDER BY project, created_at DESC";

/// Opens a connection pool.
///
/// # Errors
/// Fails when the database cannot be reached within a few seconds.
pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(url)
        .await
}

/// Applies any migrations that have not run yet.
///
/// # Errors
/// Fails when a migration cannot be applied.
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Checks that the database answers.
///
/// # Errors
/// Fails when the database is unreachable.
pub async fn ping(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}

/// Stores a report and its findings in one transaction.
///
/// # Errors
/// Fails when the database rejects the insert.
pub async fn insert_scan(pool: &PgPool, scan: &NewScan, score: u32) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO scans (id, project, git_ref, score) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(&scan.project)
        .bind(scan.git_ref.as_deref())
        .bind(i32::try_from(score).unwrap_or(i32::MAX))
        .execute(&mut *tx)
        .await?;
    if !scan.findings.is_empty() {
        let columns = FindingColumns::from_findings(&scan.findings);
        sqlx::query(INSERT_FINDINGS)
            .bind(id)
            .bind(&columns.rule_ids)
            .bind(&columns.categories)
            .bind(&columns.severities)
            .bind(&columns.messages)
            .bind(&columns.files)
            .bind(&columns.lines)
            .bind(&columns.fixes)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(id)
}

/// Lists recent scans, newest first, optionally for one project.
///
/// # Errors
/// Fails when the query fails.
pub async fn list_scans(
    pool: &PgPool,
    project: Option<&str>,
    limit: i64,
) -> Result<Vec<ScanSummary>, sqlx::Error> {
    let sql = format!(
        "{SUMMARY} WHERE ($1::text IS NULL OR s.project = $1) ORDER BY s.created_at DESC LIMIT $2"
    );
    sqlx::query_as::<_, ScanSummary>(&sql)
        .bind(project)
        .bind(limit)
        .fetch_all(pool)
        .await
}

/// Loads one scan with its findings.
///
/// # Errors
/// Fails when a query fails.
pub async fn get_scan(pool: &PgPool, id: Uuid) -> Result<Option<ScanDetail>, sqlx::Error> {
    let sql = format!("{SUMMARY} WHERE s.id = $1");
    let summary = sqlx::query_as::<_, ScanSummary>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    let Some(summary) = summary else {
        return Ok(None);
    };
    let rows = sqlx::query_as::<_, FindingRow>(SELECT_FINDINGS)
        .bind(id)
        .fetch_all(pool)
        .await?;
    let findings = rows
        .into_iter()
        .filter_map(FindingRow::into_finding)
        .collect();
    Ok(Some(ScanDetail { summary, findings }))
}

/// Lists every project with its latest score.
///
/// # Errors
/// Fails when the query fails.
pub async fn list_projects(pool: &PgPool) -> Result<Vec<ProjectSummary>, sqlx::Error> {
    sqlx::query_as::<_, ProjectSummary>(LATEST_PER_PROJECT)
        .fetch_all(pool)
        .await
}

/// Findings split into one list per column, ready for a bulk insert.
#[derive(Default)]
struct FindingColumns {
    rule_ids: Vec<String>,
    categories: Vec<String>,
    severities: Vec<String>,
    messages: Vec<String>,
    files: Vec<String>,
    lines: Vec<i32>,
    fixes: Vec<String>,
}

impl FindingColumns {
    fn from_findings(findings: &[Finding]) -> Self {
        let mut columns = Self::default();
        for finding in findings {
            columns.rule_ids.push(finding.rule_id.clone());
            columns.categories.push(to_text(&finding.category));
            columns.severities.push(to_text(&finding.severity));
            columns.messages.push(finding.message.clone());
            columns.files.push(finding.file.clone());
            columns
                .lines
                .push(i32::try_from(finding.line).unwrap_or(i32::MAX));
            columns.fixes.push(finding.fix.clone().unwrap_or_default());
        }
        columns
    }
}

#[derive(sqlx::FromRow)]
struct FindingRow {
    rule_id: String,
    category: String,
    severity: String,
    message: String,
    file: String,
    line: i32,
    fix: String,
}

impl FindingRow {
    fn into_finding(self) -> Option<Finding> {
        Some(Finding {
            rule_id: self.rule_id,
            category: from_text(&self.category)?,
            severity: from_text(&self.severity)?,
            message: self.message,
            file: self.file,
            line: u32::try_from(self.line).unwrap_or(0),
            fix: (!self.fix.is_empty()).then_some(self.fix),
        })
    }
}

/// Writes a plain enum as its lowercase text form.
fn to_text<T: Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(text)) => text,
        _ => String::new(),
    }
}

/// Reads a plain enum back from its text form.
fn from_text<T: DeserializeOwned>(text: &str) -> Option<T> {
    serde_json::from_value(serde_json::Value::String(text.to_owned())).ok()
}

#[cfg(test)]
mod tests {
    use shipcheck_core::{Category, Severity};

    use super::*;

    #[test]
    fn enums_round_trip_through_text() {
        assert_eq!(to_text(&Severity::Critical), "critical");
        assert_eq!(from_text::<Severity>("high"), Some(Severity::High));
        assert_eq!(from_text::<Category>("design"), Some(Category::Design));
        assert_eq!(from_text::<Category>("unknown"), None);
    }

    #[test]
    fn columns_line_up_with_findings() {
        let finding = Finding {
            rule_id: "LEGAL-001".to_owned(),
            category: Category::Legal,
            severity: Severity::Low,
            message: "m".to_owned(),
            file: "f".to_owned(),
            line: 7,
            fix: Some("do it".to_owned()),
        };
        let columns = FindingColumns::from_findings(&[finding.clone(), finding]);
        assert_eq!(columns.rule_ids.len(), 2);
        assert_eq!(columns.lines, vec![7, 7]);
        assert_eq!(columns.fixes[0], "do it");
    }
}
