//! Baselines: remember today's findings so that only new ones are reported.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use shipcheck_core::Finding;

use crate::error::{Error, Result};
use crate::suppress::PROJECT_FILE;

const VERSION: u32 = 1;

type LineCache = HashMap<String, Option<Vec<String>>>;

/// The findings a project has accepted for now.
///
/// A finding is remembered by its rule, its file and the text of its line, not by its
/// line number, so adding code above it does not make it look new.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Baseline {
    version: u32,
    /// Fingerprint to the number of findings with that fingerprint that are accepted.
    entries: BTreeMap<String, usize>,
}

impl Baseline {
    /// Records the given findings as accepted.
    #[must_use]
    pub fn from_findings(root: &Path, findings: &[Finding]) -> Self {
        let mut cache = LineCache::new();
        let mut entries = BTreeMap::new();
        for finding in findings {
            let key = fingerprint(root, &mut cache, finding);
            *entries.entry(key).or_insert(0) += 1;
        }
        Self {
            version: VERSION,
            entries,
        }
    }

    /// How many findings the baseline accepts.
    #[must_use]
    pub fn accepted(&self) -> usize {
        self.entries.values().sum()
    }

    /// Removes the accepted findings. Returns what is left and how many were hidden.
    #[must_use]
    pub fn filter(&self, root: &Path, findings: Vec<Finding>) -> (Vec<Finding>, usize) {
        let mut cache = LineCache::new();
        let mut remaining = self.entries.clone();
        let mut kept = Vec::new();
        let mut hidden = 0;
        for finding in findings {
            let key = fingerprint(root, &mut cache, &finding);
            match remaining.get_mut(&key) {
                Some(count) if *count > 0 => {
                    *count -= 1;
                    hidden += 1;
                }
                _ => kept.push(finding),
            }
        }
        (kept, hidden)
    }

    /// Reads a baseline file.
    pub fn load(path: &Path) -> Result<Self> {
        let origin = path.display().to_string();
        let text = fs::read_to_string(path).map_err(|source| Error::Io {
            path: origin.clone(),
            source,
        })?;
        let baseline: Self = serde_json::from_str(&text).map_err(|source| Error::Json {
            origin: origin.clone(),
            source,
        })?;
        if baseline.version != VERSION {
            return Err(Error::Invalid {
                origin,
                message: format!("unsupported baseline version {}", baseline.version),
            });
        }
        Ok(baseline)
    }

    /// Writes the baseline to a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let origin = path.display().to_string();
        let text = serde_json::to_string_pretty(self).map_err(|source| Error::Json {
            origin: origin.clone(),
            source,
        })?;
        fs::write(path, format!("{text}\n")).map_err(|source| Error::Io {
            path: origin,
            source,
        })
    }
}

fn fingerprint(root: &Path, cache: &mut LineCache, finding: &Finding) -> String {
    let anchor = line_text(root, cache, &finding.file, finding.line)
        .map_or_else(|| finding.message.clone(), |text| text.trim().to_owned());
    format!(
        "{}:{}:{:016x}",
        finding.rule_id,
        finding.file,
        fnv1a(&anchor)
    )
}

fn line_text(root: &Path, cache: &mut LineCache, file: &str, line: u32) -> Option<String> {
    if line == 0 || file == PROJECT_FILE {
        return None;
    }
    let lines = cache.entry(file.to_owned()).or_insert_with(|| {
        fs::read_to_string(root.join(file))
            .ok()
            .map(|text| text.lines().map(str::to_owned).collect())
    });
    let index = usize::try_from(line).ok()?.checked_sub(1)?;
    lines.as_ref()?.get(index).cloned()
}

/// A small, stable hash, so fingerprints do not depend on the Rust version.
fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}
