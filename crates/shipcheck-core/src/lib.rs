//! Core types shared by every Shipcheck crate.

use serde::{Deserialize, Serialize};

/// How serious a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    /// Points deducted from the project score for one finding.
    #[must_use]
    pub const fn weight(self) -> u32 {
        match self {
            Self::Info => 0,
            Self::Low => 1,
            Self::Medium => 3,
            Self::High => 6,
            Self::Critical => 10,
        }
    }
}

/// Which checklist a rule belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Legal,
    Security,
    Design,
}

/// One problem found in a scanned project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub category: Category,
    pub severity: Severity,
    pub message: String,
    pub file: String,
    pub line: u32,
    pub fix: Option<String>,
}

/// Project score from 0 to 100. Higher is better.
#[must_use]
pub fn score(findings: &[Finding]) -> u32 {
    let penalty: u32 = findings.iter().map(|f| f.severity.weight()).sum();
    100u32.saturating_sub(penalty)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(severity: Severity) -> Finding {
        Finding {
            rule_id: "TEST-001".into(),
            category: Category::Security,
            severity,
            message: "example".into(),
            file: "src/main.rs".into(),
            line: 1,
            fix: None,
        }
    }

    #[test]
    fn empty_project_scores_100() {
        assert_eq!(score(&[]), 100);
    }

    #[test]
    fn critical_finding_costs_ten_points() {
        assert_eq!(score(&[finding(Severity::Critical)]), 90);
    }

    #[test]
    fn score_never_goes_below_zero() {
        let many: Vec<Finding> = (0..50).map(|_| finding(Severity::Critical)).collect();
        assert_eq!(score(&many), 0);
    }

    #[test]
    fn finding_serializes_to_json() {
        let json = serde_json::to_string(&finding(Severity::High)).unwrap();
        assert!(json.contains("\"severity\":\"high\""));
    }
}
