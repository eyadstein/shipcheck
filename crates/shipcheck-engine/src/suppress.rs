//! Inline comments that silence findings: `shipcheck-ignore` and `shipcheck-ignore-file`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use shipcheck_core::Finding;

/// File name used by findings that belong to the whole project.
pub(crate) const PROJECT_FILE: &str = "(project)";

static DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"shipcheck-ignore(-file)?\s*:\s*([A-Za-z0-9_-]+(?:\s*,\s*[A-Za-z0-9_-]+)*)")
        .expect("directive pattern is valid")
});

/// The ignore comments found in one file.
#[derive(Default)]
struct Directives {
    file_wide: Vec<String>,
    by_line: HashMap<u32, Vec<String>>,
}

impl Directives {
    fn parse(text: &str) -> Self {
        let mut found = Self::default();
        for (index, line) in text.lines().enumerate() {
            let number = u32::try_from(index + 1).unwrap_or(u32::MAX);
            for caps in DIRECTIVE.captures_iter(line) {
                let ids = caps[2].split(',').map(|id| id.trim().to_ascii_uppercase());
                if caps.get(1).is_some() {
                    found.file_wide.extend(ids);
                } else {
                    found.by_line.entry(number).or_default().extend(ids);
                }
            }
        }
        found
    }

    /// A comment covers its own line and the line after it.
    fn covers(&self, rule_id: &str, line: u32) -> bool {
        if lists(&self.file_wide, rule_id) {
            return true;
        }
        if line == 0 {
            return false;
        }
        [line, line - 1].iter().any(|number| {
            self.by_line
                .get(number)
                .is_some_and(|ids| lists(ids, rule_id))
        })
    }
}

fn lists(ids: &[String], rule_id: &str) -> bool {
    ids.iter().any(|id| id.eq_ignore_ascii_case(rule_id))
}

fn load(root: &Path, file: &str) -> Option<Directives> {
    if file == PROJECT_FILE || file.is_empty() {
        return None;
    }
    fs::read_to_string(root.join(file))
        .ok()
        .map(|text| Directives::parse(&text))
}

/// Drops the findings that an ignore comment covers. Returns what is left and how many were dropped.
pub(crate) fn apply(root: &Path, findings: Vec<Finding>) -> (Vec<Finding>, usize) {
    let mut cache: HashMap<String, Option<Directives>> = HashMap::new();
    let mut kept = Vec::new();
    let mut dropped = 0;
    for finding in findings {
        let directives = cache
            .entry(finding.file.clone())
            .or_insert_with(|| load(root, &finding.file));
        let covered = directives
            .as_ref()
            .is_some_and(|found| found.covers(&finding.rule_id, finding.line));
        if covered {
            dropped += 1;
        } else {
            kept.push(finding);
        }
    }
    (kept, dropped)
}
