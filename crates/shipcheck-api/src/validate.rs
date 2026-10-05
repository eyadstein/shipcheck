//! Checks on incoming reports before anything is stored.

use shipcheck_core::Finding;

use crate::error::ApiError;
use crate::model::NewScan;

/// Most findings one report may hold.
pub(crate) const MAX_FINDINGS: usize = 10_000;
const MAX_PROJECT_LEN: usize = 100;
const MAX_REF_LEN: usize = 200;
const MAX_TEXT_LEN: usize = 2_000;

fn bad(message: &str) -> ApiError {
    ApiError::BadRequest(message.to_owned())
}

/// Validates a whole report.
pub(crate) fn new_scan(request: &NewScan) -> Result<(), ApiError> {
    project(&request.project)?;
    if let Some(reference) = &request.git_ref {
        git_ref(reference)?;
    }
    if request.findings.len() > MAX_FINDINGS {
        return Err(bad("a report may hold at most 10000 findings"));
    }
    request.findings.iter().try_for_each(check_finding)
}

fn project(name: &str) -> Result<(), ApiError> {
    let allowed = name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '/'));
    if name.is_empty() || name.len() > MAX_PROJECT_LEN || !allowed {
        return Err(bad(
            "project must be 1 to 100 characters: letters, digits, '.', '_', '-' or '/'",
        ));
    }
    Ok(())
}

fn git_ref(reference: &str) -> Result<(), ApiError> {
    if reference.len() > MAX_REF_LEN || reference.chars().any(char::is_control) {
        return Err(bad("git_ref must be at most 200 printable characters"));
    }
    Ok(())
}

fn check_finding(finding: &Finding) -> Result<(), ApiError> {
    if finding.rule_id.is_empty() {
        return Err(bad("rule_id must not be empty"));
    }
    let texts = [
        finding.rule_id.as_str(),
        finding.message.as_str(),
        finding.file.as_str(),
        finding.fix.as_deref().unwrap_or_default(),
    ];
    if texts.iter().any(|text| text.len() > MAX_TEXT_LEN) {
        return Err(bad("finding text fields are limited to 2000 characters"));
    }
    if texts.iter().any(|text| text.contains('\0')) {
        return Err(bad("finding text fields must not contain NUL characters"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use shipcheck_core::{Category, Severity};

    use super::*;

    fn finding() -> Finding {
        Finding {
            rule_id: "SEC-101".to_owned(),
            category: Category::Security,
            severity: Severity::High,
            message: "message".to_owned(),
            file: "app.js".to_owned(),
            line: 1,
            fix: None,
        }
    }

    fn request(project: &str, findings: Vec<Finding>) -> NewScan {
        NewScan {
            project: project.to_owned(),
            git_ref: None,
            findings,
        }
    }

    #[test]
    fn accepts_owner_repo_names() {
        assert!(new_scan(&request("eyadstein/shipcheck", vec![finding()])).is_ok());
    }

    #[test]
    fn rejects_bad_project_names() {
        let long = "a".repeat(101);
        for name in ["", "has space", "semi;colon", long.as_str()] {
            assert!(new_scan(&request(name, vec![])).is_err(), "{name:?}");
        }
    }

    #[test]
    fn rejects_too_many_findings() {
        let findings = vec![finding(); MAX_FINDINGS + 1];
        assert!(new_scan(&request("a/b", findings)).is_err());
    }

    #[test]
    fn rejects_overlong_and_nul_text() {
        let mut long = finding();
        long.message = "x".repeat(MAX_TEXT_LEN + 1);
        assert!(new_scan(&request("a/b", vec![long])).is_err());
        let mut nul = finding();
        nul.message = "a\0b".to_owned();
        assert!(new_scan(&request("a/b", vec![nul])).is_err());
    }

    #[test]
    fn rejects_control_characters_in_git_ref() {
        let mut report = request("a/b", vec![]);
        report.git_ref = Some("main\nevil".to_owned());
        assert!(new_scan(&report).is_err());
    }
}
