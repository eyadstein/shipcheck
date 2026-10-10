use std::fs;
use std::path::{Path, PathBuf};

use shipcheck_core::Finding;

use crate::rule::{Catalog, Compiled, Matcher};
use crate::walk::collect_files;

/// Largest file the scanner will read, in bytes.
const MAX_FILE_BYTES: u64 = 1_000_000;

/// Runs every rule in `catalog` against the project at `root`.
#[must_use]
pub fn scan(root: &Path, catalog: &Catalog) -> Vec<Finding> {
    let files = collect_files(root);
    let mut findings = scan_taint(root, &files);
    findings.extend(scan_design(root, &files));
    findings.extend(scan_ledger(root, &files));
    for compiled in catalog.rules() {
        match &compiled.rule.matcher {
            Matcher::LineRegex { .. } => scan_lines(root, &files, compiled, &mut findings),
            Matcher::MissingFile { any_of } => {
                if !has_any_file(&files, any_of) {
                    findings.push(finding(compiled, "(project)", 0));
                }
            }
            Matcher::RequiresPattern { .. } => {
                scan_requires(root, &files, compiled, &mut findings);
            }
        }
    }
    findings
}

fn scan_lines(root: &Path, files: &[PathBuf], compiled: &Compiled, out: &mut Vec<Finding>) {
    for path in files.iter().filter(|p| compiled.rule.applies_to(p)) {
        let Some(text) = read_text(path) else {
            continue;
        };
        let file = relative(root, path);
        for (index, line) in text.lines().enumerate() {
            if compiled.flags_line(line) {
                out.push(finding(compiled, &file, line_number(index)));
            }
        }
    }
}

fn scan_requires(root: &Path, files: &[PathBuf], compiled: &Compiled, out: &mut Vec<Finding>) {
    let (Some(when), Some(expect)) = (&compiled.regex, &compiled.expect) else {
        return;
    };
    let mut trigger: Option<(String, u32)> = None;
    for path in files.iter().filter(|p| compiled.rule.applies_to(p)) {
        let Some(text) = read_text(path) else {
            continue;
        };
        if expect.is_match(&text) {
            return;
        }
        if trigger.is_none() {
            trigger = text
                .lines()
                .position(|line| when.is_match(line))
                .map(|index| (relative(root, path), line_number(index)));
        }
    }
    if let Some((file, line)) = trigger {
        out.push(finding(compiled, &file, line));
    }
}

/// Runs syntax-tree taint analysis on every supported source file.
fn scan_taint(root: &Path, files: &[PathBuf]) -> Vec<Finding> {
    let mut out = Vec::new();
    for path in files {
        let Some(lang) = shipcheck_taint::Lang::from_path(path) else {
            continue;
        };
        let Some(text) = read_text(path) else {
            continue;
        };
        out.extend(shipcheck_taint::analyze(lang, &text, &relative(root, path)));
    }
    out
}
/// Whether a path is generated or vendored output that should not be judged.
fn is_generated(root: &Path, path: &Path) -> bool {
    let relative_path = path.strip_prefix(root).unwrap_or(path);
    let vendored = relative_path.components().any(|part| {
        matches!(
            part.as_os_str().to_str(),
            Some("node_modules" | "dist" | "build" | "vendor" | "target" | ".next")
        )
    });
    let minified = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains(".min."));
    vendored || minified
}

/// Runs the project-level design analysis on every stylesheet and page file.
fn scan_design(root: &Path, files: &[PathBuf]) -> Vec<Finding> {
    let sources: Vec<shipcheck_design::SourceFile> = files
        .iter()
        .filter(|path| shipcheck_design::is_design_file(path.as_path()))
        .filter(|path| !is_generated(root, path.as_path()))
        .filter_map(|path| {
            Some(shipcheck_design::SourceFile {
                path: relative(root, path),
                text: read_text(path)?,
            })
        })
        .collect();
    shipcheck_design::analyze(&sources)
}
/// Reads the files the disclosure ledger cares about.
fn ledger_sources(root: &Path, files: &[PathBuf]) -> Vec<shipcheck_ledger::SourceFile> {
    files
        .iter()
        .filter(|path| shipcheck_ledger::is_ledger_file(path.as_path()))
        .filter(|path| !is_generated(root, path.as_path()))
        .filter_map(|path| {
            Some(shipcheck_ledger::SourceFile {
                path: relative(root, path),
                text: read_text(path)?,
            })
        })
        .collect()
}

/// Compares what the code does with what the privacy policy says.
fn scan_ledger(root: &Path, files: &[PathBuf]) -> Vec<Finding> {
    shipcheck_ledger::analyze(&ledger_sources(root, files)).findings
}

/// Builds the disclosure ledger of the project at `root`.
#[must_use]
pub fn ledger(root: &Path) -> shipcheck_ledger::Analysis {
    let files = collect_files(root);
    shipcheck_ledger::analyze(&ledger_sources(root, &files))
}
fn line_number(index: usize) -> u32 {
    u32::try_from(index + 1).unwrap_or(u32::MAX)
}

fn has_any_file(files: &[PathBuf], names: &[String]) -> bool {
    files
        .iter()
        .filter_map(|path| path.file_name()?.to_str())
        .any(|name| names.iter().any(|wanted| wanted.eq_ignore_ascii_case(name)))
}

fn read_text(path: &Path) -> Option<String> {
    let size = fs::metadata(path).ok()?.len();
    if size > MAX_FILE_BYTES {
        return None;
    }
    fs::read_to_string(path).ok()
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn finding(compiled: &Compiled, file: &str, line: u32) -> Finding {
    let rule = &compiled.rule;
    Finding {
        rule_id: rule.id.clone(),
        category: rule.category,
        severity: rule.severity,
        message: rule.message.clone(),
        file: file.to_owned(),
        line,
        fix: rule.fix.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: &str = r"
- id: SEC-001
  category: security
  severity: high
  message: eval runs arbitrary code
  extensions: [js]
  match:
    kind: line_regex
    pattern: '\beval\('
- id: LEGAL-001
  category: legal
  severity: high
  message: No privacy policy found
  match:
    kind: missing_file
    any_of: [PRIVACY.md]
- id: TEST-REQ
  category: legal
  severity: medium
  message: Emails sent without an unsubscribe link
  extensions: [js]
  match:
    kind: requires_pattern
    when: '(?i)nodemailer'
    expect: '(?i)unsubscribe'
";

    fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::Builder::new().prefix("sc-").tempdir().unwrap();
        for (name, body) in files {
            fs::write(dir.path().join(name), body).unwrap();
        }
        dir
    }

    fn run(dir: &tempfile::TempDir) -> Vec<Finding> {
        scan(dir.path(), &Catalog::from_yaml(RULES).unwrap())
    }

    #[test]
    fn flags_matching_line_with_location() {
        let dir = project(&[
            ("app.js", "const a = 1;\neval(x);\n"),
            ("PRIVACY.md", "policy"),
        ]);
        let found = run(&dir);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].rule_id, "SEC-001");
        assert_eq!(found[0].file, "app.js");
        assert_eq!(found[0].line, 2);
    }

    #[test]
    fn reports_missing_privacy_policy() {
        let dir = project(&[("app.js", "const a = 1;\n")]);
        let found = run(&dir);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].rule_id, "LEGAL-001");
        assert_eq!(found[0].line, 0);
    }

    #[test]
    fn skips_files_with_other_extensions() {
        let dir = project(&[("notes.txt", "eval(x)\n"), ("PRIVACY.md", "policy")]);
        assert_eq!(run(&dir), Vec::<Finding>::new());
    }

    #[test]
    fn requires_pattern_flags_trigger_without_expected_text() {
        let dir = project(&[
            ("mail.js", "const m = require('nodemailer');\n"),
            ("PRIVACY.md", "policy"),
        ]);
        let found = run(&dir);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].rule_id, "TEST-REQ");
        assert_eq!(found[0].file, "mail.js");
        assert_eq!(found[0].line, 1);
    }

    #[test]
    fn requires_pattern_passes_when_expected_text_exists() {
        let dir = project(&[
            (
                "mail.js",
                "require('nodemailer'); // unsubscribe link added\n",
            ),
            ("PRIVACY.md", "policy"),
        ]);
        assert_eq!(run(&dir), Vec::<Finding>::new());
    }
}
