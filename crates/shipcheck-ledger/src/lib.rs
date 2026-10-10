//! Disclosure ledger: compares what a project's code does with what its privacy policy says.
//!
//! The ledger lists the third party services a project talks to, whether it sets
//! cookies, and which personal data it collects. Findings appear when the privacy
//! policy leaves one of those out, or says the opposite.

mod check;
mod claims;
mod evidence;
mod ledger;
mod policy;
mod render;
mod vendors;

use std::path::Path;

use shipcheck_core::Finding;

pub use ledger::{CookieUse, DataKind, DataUse, Evidence, Ledger, VendorUse};
pub use render::render_text;
pub use vendors::VendorKind;

/// One file handed to the analyzer.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Path relative to the project root, using forward slashes.
    pub path: String,
    /// File contents.
    pub text: String,
}

/// The ledger, and the findings that come from comparing it with the policy.
#[derive(Debug)]
pub struct Analysis {
    pub ledger: Ledger,
    pub findings: Vec<Finding>,
}

/// Whether the analyzer wants to read this file.
#[must_use]
pub fn is_ledger_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if evidence::is_manifest_name(name) {
        return true;
    }
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    evidence::CODE_EXTENSIONS.contains(&extension.as_str())
        || policy::POLICY_EXTENSIONS.contains(&extension.as_str())
}

/// Builds the ledger from the code, then compares it with the privacy policy if there is one.
///
/// Without a policy file the ledger is still built, but no findings are produced:
/// a missing policy is reported by the ordinary legal rules.
#[must_use]
pub fn analyze(files: &[SourceFile]) -> Analysis {
    let (policy_files, other): (Vec<&SourceFile>, Vec<&SourceFile>) = files
        .iter()
        .partition(|file| policy::is_policy_path(&file.path));
    let policies: Vec<policy::Policy> = policy_files
        .iter()
        .copied()
        .map(policy::Policy::from_file)
        .collect();
    let code: Vec<&SourceFile> = other
        .into_iter()
        .filter(|file| evidence::is_evidence_path(&file.path))
        .collect();
    let sightings = evidence::collect(&code);
    let ledger = Ledger::from_sightings(sightings, &policies);
    let findings = check::findings(&ledger, &policies, !code.is_empty());
    Analysis { ledger, findings }
}
