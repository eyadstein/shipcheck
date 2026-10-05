//! Project-level design analysis for Shipcheck.
//!
//! Finds the visual and copy patterns that make an app look machine-generated.

mod catalog;
mod checks;
mod color;
mod fonts;
mod lines;
mod project;

use std::path::Path;

use shipcheck_core::Finding;

/// File extensions the design analyzer reads.
pub const EXTENSIONS: &[&str] = &[
    "css", "scss", "html", "htm", "jsx", "tsx", "js", "ts", "vue", "svelte", "astro", "mdx",
];

/// One source file handed to the analyzer.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Path relative to the project root, using forward slashes.
    pub path: String,
    /// File contents.
    pub text: String,
}

/// Whether the analyzer wants to read this file.
#[must_use]
pub fn is_design_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            EXTENSIONS
                .iter()
                .any(|known| known.eq_ignore_ascii_case(ext))
        })
}

/// Runs every design check over the given files.
#[must_use]
pub fn analyze(files: &[SourceFile]) -> Vec<Finding> {
    let mut findings = checks::per_file(files);
    findings.extend(project::project_wide(files));
    findings
}
