use std::fs;
use std::path::Path;

use regex::Regex;
use serde::Deserialize;
use shipcheck_core::{Category, Severity};

use crate::error::{Error, Result};
use crate::walk::collect_files;

/// How a rule decides that a project has a problem.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Matcher {
    /// Flag every line that matches a regular expression.
    LineRegex { pattern: String },
    /// Flag the project when none of the listed file names exist.
    MissingFile { any_of: Vec<String> },
}

/// One rule as written in a YAML rule file.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub id: String,
    pub category: Category,
    pub severity: Severity,
    pub message: String,
    #[serde(default)]
    pub fix: Option<String>,
    /// File extensions the rule applies to. Empty means every text file.
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(rename = "match")]
    pub matcher: Matcher,
}

impl Rule {
    /// Whether this rule should run on the given file.
    #[must_use]
    pub fn applies_to(&self, path: &Path) -> bool {
        if self.extensions.is_empty() {
            return true;
        }
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| self.extensions.iter().any(|x| x.eq_ignore_ascii_case(ext)))
    }
}

/// A rule paired with its compiled pattern.
#[derive(Debug)]
pub struct Compiled {
    pub rule: Rule,
    pub regex: Option<Regex>,
}

impl Compiled {
    fn new(rule: Rule) -> Result<Self> {
        let regex = match &rule.matcher {
            Matcher::LineRegex { pattern } => {
                let compiled = Regex::new(pattern).map_err(|source| Error::Pattern {
                    id: rule.id.clone(),
                    source,
                })?;
                Some(compiled)
            }
            Matcher::MissingFile { .. } => None,
        };
        Ok(Self { rule, regex })
    }
}

/// All rules loaded from disk, with patterns compiled.
#[derive(Debug, Default)]
pub struct Catalog {
    rules: Vec<Compiled>,
}

impl Catalog {
    /// Loads every `.yml` and `.yaml` file under `dir`.
    ///
    /// # Errors
    /// Fails when the directory is missing, a file cannot be read, a file is
    /// not valid YAML, or a rule holds an invalid pattern.
    pub fn load(dir: &Path) -> Result<Self> {
        if !dir.is_dir() {
            return Err(Error::MissingRulesDir(dir.display().to_string()));
        }
        let mut rules = Vec::new();
        for path in collect_files(dir).iter().filter(|p| is_rule_file(p)) {
            let origin = path.display().to_string();
            let text = fs::read_to_string(path).map_err(|source| Error::Io {
                path: origin.clone(),
                source,
            })?;
            rules.extend(parse(&text, &origin)?);
        }
        Ok(Self { rules })
    }

    /// Builds a catalog from YAML text.
    ///
    /// # Errors
    /// Fails when the text is not valid YAML or a pattern does not compile.
    pub fn from_yaml(text: &str) -> Result<Self> {
        Ok(Self {
            rules: parse(text, "<inline>")?,
        })
    }

    /// The compiled rules, in load order.
    #[must_use]
    pub fn rules(&self) -> &[Compiled] {
        &self.rules
    }
}

fn is_rule_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml"))
}

fn parse(text: &str, origin: &str) -> Result<Vec<Compiled>> {
    let rules: Vec<Rule> = serde_yaml::from_str(text).map_err(|source| Error::Yaml {
        origin: origin.to_owned(),
        source,
    })?;
    rules.into_iter().map(Compiled::new).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_a_valid_rule() {
        let yaml = "- id: DES-001\n  category: design\n  severity: low\n  message: example\n  match:\n    kind: line_regex\n    pattern: 'foo'\n";
        let catalog = Catalog::from_yaml(yaml).unwrap();
        assert_eq!(catalog.rules().len(), 1);
        assert!(catalog.rules()[0].regex.is_some());
    }

    #[test]
    fn invalid_pattern_is_reported() {
        let yaml = "- id: BAD\n  category: design\n  severity: low\n  message: x\n  match:\n    kind: line_regex\n    pattern: '('\n";
        assert!(matches!(
            Catalog::from_yaml(yaml),
            Err(Error::Pattern { .. })
        ));
    }
}
