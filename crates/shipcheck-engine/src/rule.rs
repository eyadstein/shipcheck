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
    /// Flag every line that matches `pattern`, unless it also matches `unless`.
    LineRegex {
        pattern: String,
        #[serde(default)]
        unless: Option<String>,
    },
    /// Flag the project when none of the listed file names exist.
    MissingFile { any_of: Vec<String> },
    /// Flag the project when `when` matches somewhere but `expect` matches nowhere.
    RequiresPattern { when: String, expect: String },
}

/// Sample lines that document and test a rule.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Examples {
    /// Lines the rule must flag.
    #[serde(default)]
    pub flag: Vec<String>,
    /// Lines the rule must leave alone.
    #[serde(default)]
    pub pass: Vec<String>,
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
    #[serde(default)]
    pub examples: Examples,
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

/// A rule paired with its compiled patterns.
#[derive(Debug)]
pub struct Compiled {
    pub rule: Rule,
    /// Main pattern: `pattern` for `line_regex`, `when` for `requires_pattern`.
    pub regex: Option<Regex>,
    /// The `expect` pattern of a `requires_pattern` rule.
    pub expect: Option<Regex>,
    /// The `unless` pattern of a `line_regex` rule.
    pub unless: Option<Regex>,
}

impl Compiled {
    fn new(rule: Rule) -> Result<Self> {
        let id = rule.id.as_str();
        let (regex, expect, unless) = match &rule.matcher {
            Matcher::LineRegex { pattern, unless } => (
                Some(compile(id, pattern)?),
                None,
                unless
                    .as_deref()
                    .map(|text| compile(id, text))
                    .transpose()?,
            ),
            Matcher::MissingFile { .. } => (None, None, None),
            Matcher::RequiresPattern { when, expect } => {
                (Some(compile(id, when)?), Some(compile(id, expect)?), None)
            }
        };
        Ok(Self {
            rule,
            regex,
            expect,
            unless,
        })
    }

    /// Whether the rule's main pattern fires on a single line.
    #[must_use]
    pub fn flags_line(&self, line: &str) -> bool {
        let Some(regex) = &self.regex else {
            return false;
        };
        regex.is_match(line) && !self.unless.as_ref().is_some_and(|skip| skip.is_match(line))
    }

    /// Checks the rule's examples against its main pattern.
    /// Returns one message per example that behaves wrongly.
    #[must_use]
    pub fn check_examples(&self) -> Vec<String> {
        let id = &self.rule.id;
        let missed = self
            .rule
            .examples
            .flag
            .iter()
            .filter(|line| !self.flags_line(line))
            .map(|line| format!("{id}: should flag {line:?}"));
        let wrong = self
            .rule
            .examples
            .pass
            .iter()
            .filter(|line| self.flags_line(line))
            .map(|line| format!("{id}: should not flag {line:?}"));
        missed.chain(wrong).collect()
    }
}

fn compile(id: &str, pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(|source| Error::Pattern {
        id: id.to_owned(),
        source,
    })
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

    /// Runs every rule's examples and returns all failures.
    #[must_use]
    pub fn check_examples(&self) -> Vec<String> {
        self.rules
            .iter()
            .flat_map(Compiled::check_examples)
            .collect()
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

    #[test]
    fn unless_pattern_suppresses_a_match() {
        let yaml = "- id: A11Y\n  category: legal\n  severity: low\n  message: x\n  match:\n    kind: line_regex\n    pattern: '<img'\n    unless: 'alt='\n";
        let catalog = Catalog::from_yaml(yaml).unwrap();
        let compiled = &catalog.rules()[0];
        assert!(compiled.flags_line("<img src=x>"));
        assert!(!compiled.flags_line("<img src=x alt=y>"));
    }
}
