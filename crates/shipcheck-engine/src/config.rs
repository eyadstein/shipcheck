//! Project settings read from `shipcheck.yml`.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};

const FILE_NAMES: [&str; 2] = ["shipcheck.yml", ".shipcheck.yml"];

/// Settings a project keeps next to its code.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    /// Rule ids that are switched off for the whole project.
    #[serde(default)]
    pub ignore_rules: Vec<String>,
    /// Path prefixes whose findings are ignored.
    #[serde(default)]
    pub ignore_paths: Vec<String>,
    /// Minimum score, used when the command line does not give one.
    #[serde(default)]
    pub fail_under: Option<u32>,
}

impl ProjectConfig {
    /// Loads the settings from the project root. A missing file means no settings.
    pub fn load(root: &Path) -> Result<Self> {
        for name in FILE_NAMES {
            let path = root.join(name);
            if !path.is_file() {
                continue;
            }
            let origin = path.display().to_string();
            let text = fs::read_to_string(&path).map_err(|source| Error::Io {
                path: origin.clone(),
                source,
            })?;
            return Self::from_yaml(&text, &origin);
        }
        Ok(Self::default())
    }

    /// Reads settings from YAML text. `origin` names the source in error messages.
    pub fn from_yaml(text: &str, origin: &str) -> Result<Self> {
        if text.trim().is_empty() {
            return Ok(Self::default());
        }
        let config: Self = serde_yaml::from_str(text).map_err(|source| Error::Yaml {
            origin: origin.to_owned(),
            source,
        })?;
        if config.fail_under.is_some_and(|value| value > 100) {
            return Err(Error::Invalid {
                origin: origin.to_owned(),
                message: "fail_under must be between 0 and 100".to_owned(),
            });
        }
        Ok(config)
    }

    /// Whether a finding of `rule_id` in `file` is switched off by the settings.
    #[must_use]
    pub fn ignores(&self, rule_id: &str, file: &str) -> bool {
        self.ignore_rules.iter().any(|id| id == rule_id)
            || self
                .ignore_paths
                .iter()
                .any(|prefix| path_is_under(file, prefix))
    }
}

fn path_is_under(file: &str, prefix: &str) -> bool {
    let prefix = prefix.trim_start_matches("./").trim_end_matches('/');
    if prefix.is_empty() {
        return false;
    }
    file == prefix
        || file
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('/'))
}
