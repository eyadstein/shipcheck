//! What happens to findings after a scan: project settings, inline comments, the baseline.

use std::path::Path;

use shipcheck_core::Finding;

use crate::baseline::Baseline;
use crate::config::ProjectConfig;
use crate::suppress;

/// The findings that are left, and how many were set aside on the way.
#[derive(Debug)]
pub struct Review {
    pub findings: Vec<Finding>,
    /// Switched off by `shipcheck.yml`.
    pub ignored: usize,
    /// Silenced by `shipcheck-ignore` comments in the code.
    pub suppressed: usize,
    /// Already accepted by the baseline.
    pub baselined: usize,
}

/// Applies the project settings, then the inline comments, then the baseline.
#[must_use]
pub fn review(
    root: &Path,
    findings: Vec<Finding>,
    config: &ProjectConfig,
    baseline: Option<&Baseline>,
) -> Review {
    let before = findings.len();
    let findings: Vec<Finding> = findings
        .into_iter()
        .filter(|finding| !config.ignores(&finding.rule_id, &finding.file))
        .collect();
    let ignored = before - findings.len();
    let (findings, suppressed) = suppress::apply(root, findings);
    let (findings, baselined) = match baseline {
        Some(baseline) => baseline.filter(root, findings),
        None => (findings, 0),
    };
    Review {
        findings,
        ignored,
        suppressed,
        baselined,
    }
}
